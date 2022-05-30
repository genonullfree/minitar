use std::env;
use std::fs;
use std::fs::File;
use std::fs::Metadata;
use std::io::{Error, ErrorKind};
use std::os::unix::prelude::FileTypeExt;
use std::path::Path;
use std::str;
use std::string::String;

use deku::prelude::*;

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;

/// Represents the different types of files that can be encoded in a tar file.
#[repr(u8)]
pub enum FileType {
    Normal = 0x30,
    Hard = 0x31,
    Symbolic = 0x32,
    Character = 0x33,
    Block = 0x34,
    Directory = 0x35,
    FIFO = 0x36,
    Unknown = 0x00,
}

/// Encompases all the data that is contained within a Tar File Header.
#[derive(Clone, Copy, DekuRead, DekuWrite, PartialEq)]
#[deku(endian = "little")]
pub struct TarHeader {
    file_name: [u8; 100],
    file_mode: [u8; 8],
    own_user: [u8; 8],
    own_group: [u8; 8],
    file_size: [u8; 12],
    mod_time: [u8; 12],
    header_checksum: [u8; 8],
    link_indicator: [u8; 1],
    link_name: [u8; 100],
    ustar_magic: [u8; 6],
    ustar_version: [u8; 2],
    own_user_name: [u8; 32],
    own_group_name: [u8; 32],
    device_major: [u8; 8],
    device_minor: [u8; 8],
    file_prefix: [u8; 155],
    reserved: [u8; 12],
}

impl Default for TarHeader {
    fn default() -> TarHeader {
        TarHeader {
            file_name: [0; 100],
            file_mode: [0; 8],
            own_user: [0; 8],
            own_group: [0; 8],
            file_size: [0; 12],
            mod_time: [0; 12],
            header_checksum: [0; 8],
            link_indicator: [0; 1],
            link_name: [0; 100],
            ustar_magic: [0; 6],
            ustar_version: [0; 2],
            own_user_name: [0; 32],
            own_group_name: [0; 32],
            device_major: [0; 8],
            device_minor: [0; 8],
            file_prefix: [0; 155],
            reserved: [0; 12],
        }
    }
}

impl TarHeader {
    /// Validates that the magic value received matches the magic value required in the Tar specification.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarHeader;
    /// let header = TarHeader::default();
    /// if !header.validate_magic() {
    ///     println!("Magic value is invalid");
    /// }
    /// ```
    pub fn validate_magic(self) -> bool {
        self.ustar_magic == "ustar ".as_bytes()
    }

    /// Validates the header checksum computes to the expected value.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarHeader;
    /// let header = TarHeader::default();
    /// if header.validate_checksum() {
    ///     println!("Checksum is valid");
    /// }
    /// ```
    pub fn validate_checksum(self) -> bool {
        let mut test = self;
        let mut new = [0x20u8; 8];
        test.header_checksum.copy_from_slice(&[0x20; 8]);

        let tmp = format!("{:06o}\x00", checksum_header(test));
        new[..tmp.len()].copy_from_slice(tmp.as_bytes());

        self.header_checksum == new
    }
}

/// Contains a tar representation of a file.
#[derive(Clone, Default)]
pub struct TarNode {
    header: TarHeader,
    data: Vec<[u8; 512]>,
}

impl TarNode {
    pub fn write<T: std::io::Write>(self, mut input: T) -> Result<(), Error> {
        input.write_all(&self.header.to_bytes().unwrap())?;
        for d in self.data {
            input.write_all(&d)?;
        }

        Ok(())
    }

    pub fn read<T: std::io::Read>(mut input: T) -> Result<TarNode, Error> {
        let mut h = vec![0u8; 512];
        input.read_exact(&mut h)?;

        let (_, header) = TarHeader::from_bytes((&h, 0)).unwrap();
        if header == TarHeader::default() {
            return Err(Error::new(ErrorKind::InvalidData, "End of tar"));
        }
        if !header.validate_magic() {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic"));
        }
        if !header.validate_checksum() {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid checksum"));
        }

        let chunks = (oct_to_dec(&header.file_size) / 512) + 1;
        Ok(TarNode {
            header,
            data: TarNode::chunk_file(&mut input, Some(chunks))?,
        })
    }

    fn read_file_to_tar(filename: String) -> Result<TarNode, Error> {
        let mut file = File::open(&filename)?;
        Ok(TarNode {
            header: generate_header(&filename),
            data: TarNode::chunk_file(&mut file, None)?,
        })
    }

    fn chunk_file<T: std::io::Read>(
        file: &mut T,
        max_chunks: Option<usize>,
    ) -> Result<Vec<[u8; 512]>, Error> {
        /* Extract the file data from the tar file */
        let mut out = Vec::<[u8; 512]>::new();
        let mut n = if let Some(max) = max_chunks {
            max
        } else {
            usize::MAX
        };

        loop {
            /* Carve out 512 bytes at a time */
            let mut buf: [u8; 512] = [0; 512];
            let len = file.read(&mut buf).expect("Failed to read");

            n -= 1;

            /* If read len == 0, we've hit the EOF */
            if len == 0 || n == 0 {
                break;
            }

            /* Save this chunk */
            out.push(buf);
        }
        Ok(out)
    }
}

/// Contains the vector of files that represent a tar file.
#[derive(Clone, Default)]
pub struct TarFile {
    file: Vec<TarNode>,
}

impl TarFile {
    pub fn write<T: std::io::Write + Copy>(self, mut input: T) -> Result<(), Error> {
        for f in self.file {
            f.write(input)?;
        }

        /* Complete the write with 18 blocks of 512 ``0x00`` bytes per the specification */
        input.write_all(&[0; 9216])?;

        Ok(())
    }

    pub fn new(filename: String) -> Result<Self, Error> {
        Ok(TarFile {
            file: vec![TarNode::read_file_to_tar(filename)?],
        })
    }

    pub fn append(&mut self, filename: String) -> Result<(), Error> {
        self.file.push(TarNode::read_file_to_tar(filename).unwrap());

        Ok(())
    }

    pub fn open(filename: String) -> Result<Self, Error> {
        let file = File::open(&filename).unwrap();
        let mut out = TarFile {
            file: Vec::<TarNode>::new(),
        };

        while let Ok(t) = TarNode::read(&file) {
            out.file.push(t);
        }

        Ok(out)
    }
}

fn get_file_type(file_type: &dyn FileTypeExt, meta: &Metadata) -> [u8; 1] {
    if file_type.is_fifo() {
        return [0x36];
    } else if file_type.is_char_device() {
        return [0x33];
    } else if file_type.is_block_device() {
        return [0x34];
    } else if meta.is_dir() {
        return [0x35];
    }
    /* Normal file meta.is_file() */
    [0x30]
}

//Incomplete
fn generate_header(filename: &String) -> TarHeader {
    let mut head = TarHeader::default();
    let meta = fs::metadata(&filename).expect("Failed to get file metadata");
    let file_type = meta.file_type();
    let name = Path::new(&filename)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    /* Fill in metadata */
    head.file_name[..name.len()].copy_from_slice(name.as_bytes());
    let mode = format!("{:07o}", (meta.st_mode() & 0o777));
    head.file_mode[..mode.len()].copy_from_slice(mode.as_bytes());
    let user = format!("{:07o}", meta.st_uid());
    head.own_user[..user.len()].copy_from_slice(user.as_bytes());
    let group = format!("{:07o}", meta.st_gid());
    head.own_group[..group.len()].copy_from_slice(group.as_bytes());
    let size = format!("{:011o}", meta.st_size());
    head.file_size[..size.len()].copy_from_slice(size.as_bytes());
    let mtime = format!("{:011o}", meta.st_mtime());
    head.mod_time[..mtime.len()].copy_from_slice(mtime.as_bytes());
    let checksum: [u8; 8] = [0x20; 8];
    head.header_checksum.copy_from_slice(&checksum);
    head.link_indicator = get_file_type(&file_type, &meta);
    /* Get link_name via fs::symlink_metadata */
    // let link_name ...default '' ...fs::symlink_metadata
    let magic: [u8; 6] = [0x75, 0x73, 0x74, 0x61, 0x72, 0x20];
    head.ustar_magic[..magic.len()].copy_from_slice(&magic);
    let version: [u8; 2] = [0x20, 0x00];
    head.ustar_version[..version.len()].copy_from_slice(&version);
    /* TODO: Find better way to get username */
    let key = "USER";
    if let Ok(val) = env::var(key) {
        head.own_user_name[..val.len()].copy_from_slice(val.as_bytes())
    }
    /* TODO: Find way to get groupname */
    /* TODO: Get major and minor device numbers when applicable
    let major = format!("{:07o}", meta.st_dev());
    head.device_major[..major.len()].copy_from_slice(major.as_bytes());
    let minor = format!("{:07o}", meta.st_rdev());
    head.device_minor[..minor.len()].copy_from_slice(minor.as_bytes());
    */

    let checksum = format!("{:06o}\x00", checksum_header(head));
    head.header_checksum[..checksum.len()].copy_from_slice(checksum.as_bytes());

    head
}

fn checksum_header(tar: TarHeader) -> u64 {
    let out = tar.to_bytes().unwrap();
    let mut checksum: u64 = 0;
    for i in out {
        checksum += i as u64;
    }
    checksum
}

fn oct_to_dec(input: &[u8]) -> usize {
    /* Convert the &[u8] to string and remove the null byte */
    let mut s = str::from_utf8(input)
        .expect("Cannot convert utf8")
        .to_string();
    s.pop();

    /* Convert to usize from octal */
    usize::from_str_radix(&s, 8).unwrap_or_else(|_| panic!("Cannot convert oct to decimal: {}", &s))
}
