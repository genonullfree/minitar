use std::env;
use std::fs;
use std::fs::File;
use std::fs::Metadata;
use std::io::BufReader;
use std::os::unix::prelude::FileTypeExt;
use std::str;
use std::string::String;

use deku::prelude::*;

use crate::error::TarError;

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;

/// Represents the different types of files that can be encoded in a tar file.
#[repr(u8)]
pub enum FileType {
    Normal = 0x30,
    Hard = 0x31,
    Sym = 0x32,
    Char = 0x33,
    Block = 0x34,
    Dir = 0x35,
    FIFO = 0x36,
    Unknown = 0x00,
}

/// Contains the representation of a Tar file header.
#[derive(Clone, Copy, Debug, DekuRead, DekuWrite, PartialEq)]
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
    /// if header.validate_checksum().unwrap() {
    ///     println!("Checksum is valid");
    /// }
    /// ```
    pub fn validate_checksum(self) -> Result<bool, TarError> {
        let mut test = self;
        let mut new = [0x20u8; 8];
        test.header_checksum.copy_from_slice(&[0x20; 8]);

        let tmp = format!("{:06o}\x00", test.calc_checksum()?);
        new[..tmp.len()].copy_from_slice(tmp.as_bytes());

        Ok(self.header_checksum == new)
    }

    /// Updates the header checksum value.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarHeader;
    /// let mut header = TarHeader::default();
    ///
    /// /* Fill in header information */
    ///
    /// header.update_checksum();
    /// ```
    pub fn update_checksum(&mut self) -> Result<(), TarError> {
        let checksum = format!("{:06o}\x00", self.calc_checksum()?);
        self.header_checksum[..checksum.len()].copy_from_slice(checksum.as_bytes());
        Ok(())
    }

    fn calc_checksum(self) -> Result<usize, TarError> {
        let out = self.to_bytes()?;
        let mut checksum = 0;
        for i in out {
            checksum += i as usize;
        }
        Ok(checksum)
    }
}

/// Contains a tar representation of a file.
#[derive(Clone, Debug, Default)]
pub struct TarNode {
    header: TarHeader,
    data: Vec<[u8; 512]>,
}

impl TarNode {
    /// Write out a single file within the tar to a file or something with a ``std::io::Write`` trait.
    pub fn write<T: std::io::Write>(self, mut input: T) -> Result<usize, TarError> {
        input.write_all(&self.header.to_bytes()?)?;
        let mut written = 512;
        for d in self.data {
            input.write_all(&d)?;
            written += d.len();
        }

        Ok(written)
    }

    /// Read a TarNode in from a file or something with a ``std::io::Read`` trait.
    pub fn read<T: std::io::Read>(mut input: T) -> Result<TarNode, TarError> {
        let mut h = vec![0u8; 512];
        input.read_exact(&mut h)?;

        let (_, header) = TarHeader::from_bytes((&h, 0))?;
        if header == TarHeader::default() {
            return Err(TarError::EndOfTar);
        }
        if !header.validate_magic() {
            return Err(TarError::InvalidMagic);
        }
        if !header.validate_checksum()? {
            return Err(TarError::InvalidChecksum);
        }

        let chunks = (oct_to_dec(&header.file_size)? / 512) + 1;
        Ok(TarNode {
            header,
            data: TarNode::chunk_file(&mut input, Some(chunks))?,
        })
    }

    /// Open and read a file from the ``filename`` argument to a TarNode.
    fn read_file_to_tar(filename: String) -> Result<TarNode, TarError> {
        let header = generate_header(&filename)?;
        if header.link_indicator[0] != FileType::Normal as u8 {
            return Ok(TarNode {
                header,
                data: Vec::<[u8; 512]>::new(),
            });
        }

        let file = File::open(&filename)?;
        let mut reader = BufReader::new(file);
        Ok(TarNode {
            header,
            data: TarNode::chunk_file(&mut reader, None)?,
        })
    }

    /// Read in and split a file into ``512`` byte chunks.
    fn chunk_file<T: std::io::Read>(
        file: &mut T,
        max_chunks: Option<usize>,
    ) -> Result<Vec<[u8; 512]>, TarError> {
        /* Extract the file data from the tar file */
        let mut out = Vec::<[u8; 512]>::new();
        let mut n = if let Some(max) = max_chunks {
            max
        } else {
            usize::MAX
        };

        /* Carve out 512 bytes at a time */
        let mut buf: [u8; 512] = [0; 512];
        loop {
            let len = file.read(&mut buf)?;

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
#[derive(Clone, Debug, Default)]
pub struct TarFile {
    file: Vec<TarNode>,
}

impl TarFile {
    /// Write out a vector of `TarNodes` to a file or something that implements ``std::io::Write`` and ``std::io::Copy``.
    ///
    /// # Example
    ///
    /// ```
    /// use std::fs::File;
    /// use minitar::tar::TarFile;
    ///
    /// let data = TarFile::new("test/1.txt".to_string()).unwrap();
    ///
    /// let out = File::create("test/2.tar".to_string()).unwrap();
    /// data.write(&out).unwrap();
    /// ```
    pub fn write<T: std::io::Write + Copy>(self, mut input: T) -> Result<usize, TarError> {
        let mut written = 0;
        for f in self.file.clone() {
            written += f.write(input)?;
        }

        /* Complete the write with 18 blocks of 512 ``0x00`` bytes per the specification */
        if !self.file.is_empty() {
            input.write_all(&[0; 9216])?;
            written += 9216;
        }

        Ok(written)
    }

    /// Create a new `TarFile` struct and initialize it with a `filename` file. This will read in the file to
    /// the `TarFile` struct as a `TarNode`.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarFile;
    ///
    /// let data = TarFile::new("test/1.txt".to_string()).unwrap();
    /// ```
    pub fn new(filename: String) -> Result<Self, TarError> {
        Ok(TarFile {
            file: vec![TarNode::read_file_to_tar(filename)?],
        })
    }

    /// Append another file to the `TarFile.file` vector. This adds a file to the internal representation of the tar file.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarFile;
    ///
    /// let mut data = TarFile::new("test/1.txt".to_string()).unwrap();
    /// data.append("test/1.txt".to_string()).unwrap();
    /// ```
    pub fn append(&mut self, filename: String) -> Result<(), TarError> {
        self.file.push(TarNode::read_file_to_tar(filename)?);

        Ok(())
    }

    /// Open and load an external tar file into the internal `TarFile` struct. This parses and loads up all the files
    /// contained within the external tar file.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarFile;
    ///
    /// TarFile::open("test/1.tar".to_string()).unwrap();
    /// ```
    pub fn open(filename: String) -> Result<Self, TarError> {
        let file = File::open(&filename)?;
        let mut reader = BufReader::new(file);
        let mut out = TarFile {
            file: Vec::<TarNode>::new(),
        };

        while let Ok(t) = TarNode::read(&mut reader) {
            out.file.push(t);
        }

        Ok(out)
    }

    /// Remove the first file from the Tar that matches the filename and path.
    ///
    /// # Example
    ///
    /// ```
    /// use minitar::tar::TarFile;
    ///
    /// let mut data = TarFile::new("test/1.tar".to_string()).unwrap();
    /// data.remove("test/1.tar".to_string()).unwrap();
    /// ```
    pub fn remove(&mut self, filename: String) -> Result<bool, TarError> {
        let mut name = [0u8; 100];
        name[..filename.len()].copy_from_slice(filename.as_bytes());
        if let Some(i) = &self.file.iter().position(|x| x.header.file_name == name) {
            self.file.remove(*i);
            return Ok(true);
        }

        Ok(false)
    }
}

fn get_file_type(meta: &Metadata) -> u8 {
    if meta.is_dir() {
        return FileType::Dir as u8;
    }

    let file_type = meta.file_type();
    if file_type.is_fifo() {
        return FileType::FIFO as u8;
    } else if file_type.is_char_device() {
        return FileType::Char as u8;
    } else if file_type.is_block_device() {
        return FileType::Block as u8;
    } else if file_type.is_fifo() {
        return FileType::FIFO as u8;
    } else if file_type.is_symlink() {
        return FileType::Sym as u8;
    } else if file_type.is_file() {
        return FileType::Normal as u8;
    }

    FileType::Unknown as u8
}

fn generate_header(filename: &String) -> Result<TarHeader, TarError> {
    let mut head = TarHeader::default();
    let meta = fs::symlink_metadata(&filename)?;

    /* Fill in metadata */
    head.file_name[..filename.len()].copy_from_slice(filename.as_bytes());
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

    /* Get the file type and conditional metadata */
    head.link_indicator[0] = get_file_type(&meta);
    if head.link_indicator[0] == FileType::Sym as u8 {
        let link = fs::read_link(&filename)?.to_str().unwrap().to_string();
        head.link_name[..link.len()].copy_from_slice(link.as_bytes());
    } else if head.link_indicator[0] == FileType::Block as u8 {
        let major = format!("{:07o}", meta.st_dev());
        head.device_major[..major.len()].copy_from_slice(major.as_bytes());
        let minor = format!("{:07o}", meta.st_rdev());
        head.device_minor[..minor.len()].copy_from_slice(minor.as_bytes());
    }

    /* TODO: Find better way to get username */
    let key = "USER";
    if let Ok(val) = env::var(key) {
        head.own_user_name[..val.len()].copy_from_slice(val.as_bytes())
    }
    /* TODO: Find way to get groupname */

    /* Set USTAR magic and version info */
    head.ustar_magic = [0x75, 0x73, 0x74, 0x61, 0x72, 0x20];
    head.ustar_version = [0x20, 0x00];
    head.header_checksum = [0x20; 8];

    /* Update the header checksum value */
    head.update_checksum()?;

    Ok(head)
}

fn oct_to_dec(input: &[u8]) -> Result<usize, TarError> {
    /* Convert the &[u8] to string and remove the null byte */
    let mut s = str::from_utf8(input)?.to_string();
    s.pop();

    /* Convert to usize from octal */
    Ok(usize::from_str_radix(&s, 8)?)
}
