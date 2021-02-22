#![allow(non_camel_case_types)]
#![allow(non_camel_case)]

use std::fs::File;
use std::str;
use std::io::Read;
use std::io::Write;
use std::fs;
use std::fs::Metadata;
use std::os::linux::fs::MetadataExt;
use std::os::unix::prelude::FileTypeExt;
use std::string::String;

#[repr(u8)]
pub enum type_flag {
    Normal = 0x30,
    Hard = 0x31,
    Symbolic = 0x32,
    Character = 0x33,
    Block = 0x34,
    Directory = 0x35,
    FIFO = 0x36,
    Unknown = 0x00,
}

#[derive(Clone)]
pub struct tar_header {
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

impl Default for tar_header {
    fn default() -> tar_header {
        tar_header {
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

#[derive(Clone)]
pub struct tar_node {
    header: tar_header,
    data: Vec<[u8; 512]>,
}

impl Default for tar_node {
    fn default() -> tar_node {
        tar_node {
            header: tar_header::default(),
            data: Vec::<[u8; 512]>::new(),
        }
    }
}

//Incomplete
pub fn tar_append(filename: File, tar: &mut Vec<tar_node>) {}

//Incomplete
pub fn file_read(filename: String) -> Vec<tar_node> {
    /* TODO: Use for opening regular files */
    let mut file = File::open(&filename).expect("Could not open file");
    let mut tar = Vec::<tar_node>::new();

    tar.push(tar_node {
        header: generate_header(&filename),
        data: read_file(&mut file),
    });

    tar
}

pub fn tar_read(filename: String) -> Vec<tar_node> {
    /* Open and ingest a tar file for processing */
    let mut file = File::open(filename).expect("Could not open file");

    let out = ingest(&mut file);

    out
}

pub fn tar_write(filename: String, tar: &mut Vec<tar_node>) {
    /* Append the end 0x00 bytes for the file footer */
    append_end(tar);

    /* Serialize the tar data */
    let flat = serialize(&tar);

    /* Create and write the tar data to file */
    let mut file = File::create(filename).expect("Error creating file");
    file.write_all(&flat).expect("Error writing file");
    file.flush().expect("Error flushing file");
}

//Incomplete
fn ingest(filename: &mut File) -> Vec<tar_node> {
    /* TODO: While (read_tar_header), get next file */
    let mut tar = Vec::<tar_node>::new();
    match read_tar_header(filename) {
        Some(n) => {
            let o = oct_to_dec(&n.file_size);
            tar.push(tar_node {
                header: n,
                data: extract_file(filename, o),
            });
        }
        _ => {}
    };
    tar
}

fn validate_magic(header: &tar_header) -> bool {
    /* Validate magic header value with magic value */
    let magic: [u8; 6] = [ 0x75, 0x73, 0x74, 0x61, 0x72, 0x20 ];
    header.ustar_magic == magic
}

fn get_file_type(file_type: &FileTypeExt, meta: &Metadata) -> [u8; 1] {
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
    return [0x30];
}

//Incomplete
fn generate_header(filename: &String) -> tar_header {
    let mut head = tar_header::default();
    let meta = fs::metadata(&filename).expect("Failed to get file metadata");
    let file_type = meta.file_type();

    /* Fill in metadata */
    head.file_name[..filename.len()].copy_from_slice(&filename.as_bytes());
    let mode = format!("{:07o}", meta.st_mode());
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
    // let linktype ...default '0'
    head.link_indicator = get_file_type(&file_type, &meta);
    // let link_name ...default '' ...fs::symlink_metadata
    let magic: [u8; 6] = [ 0x75, 0x73, 0x74, 0x61, 0x72, 0x20 ];
    head.ustar_magic[..magic.len()].copy_from_slice(&magic);
    let version: [u8; 2] = [ 0x20, 0x00 ];
    head.ustar_version[..version.len()].copy_from_slice(&version);
    // let user_name
    // let group_name
    /*
    let major = format!("{:07o}", meta.st_dev());
    head.device_major[..major.len()].copy_from_slice(major.as_bytes());
    let minor = format!("{:07o}", meta.st_rdev());
    head.device_minor[..minor.len()].copy_from_slice(minor.as_bytes());
    */

    let checksum = format!("{:06}\x00", checksum_header(head.clone()));
    head.header_checksum[..checksum.len()].copy_from_slice(&checksum.as_bytes());

    head
}

fn read_tar_header(filename: &mut File) -> Option<tar_header> {
    /* Create a new tar_header struct and read in the values */
    let mut header: tar_header = tar_header::default();
    filename.read_exact(&mut header.file_name).expect("Error reading file_name");
    filename.read_exact(&mut header.file_mode).expect("Error reading file_mode");
    filename.read_exact(&mut header.own_user).expect("Error reading own_user");
    filename.read_exact(&mut header.own_group).expect("Error reading own_group");
    filename.read_exact(&mut header.file_size).expect("Error reading file_size");
    filename.read_exact(&mut header.mod_time).expect("Error reading mod_time");
    filename.read_exact(&mut header.header_checksum).expect("Error reading header_checksum");
    filename.read_exact(&mut header.link_indicator).expect("Error reading link_indicator");
    filename.read_exact(&mut header.link_name).expect("Error reading link_name");
    filename.read_exact(&mut header.ustar_magic).expect("Error reading ustar_magic");
    filename.read_exact(&mut header.ustar_version).expect("Error reading ustar_version");
    filename.read_exact(&mut header.own_user_name).expect("Error reading own_user_name");
    filename.read_exact(&mut header.own_group_name).expect("Error reading own_group_name");
    filename.read_exact(&mut header.device_major).expect("Error reading device_major");
    filename.read_exact(&mut header.device_minor).expect("Error reading device_minor");
    filename.read_exact(&mut header.file_prefix).expect("Error reading file_prefix");
    filename.read_exact(&mut header.reserved).expect("Error reading reserved");

    /* Validate the header magic value */
    if validate_magic(&header) {
        return Some(header);
    }

    None
}

fn read_file<T: std::io::Read>(file: &mut T) -> Vec<[u8; 512]> {
    /* Extract the file data from the tar file */
    let mut out = Vec::<[u8; 512]>::new();

    loop {
        /* Carve out 512 bytes at a time */
        let mut buf: [u8; 512] = [0; 512];
        let len = file.read(&mut buf).expect("Failed to read");

        /* If read len == 0, we've hit the EOF */
        if len == 0 {
            break;
        }

        /* Save this chunk */
        out.push(buf);
    }
    out
}

fn extract_file<T: std::io::Read>(file: &mut T, file_size: usize) -> Vec<[u8; 512]> {
    /* Extract the file data from the tar file */
    let mut out = Vec::<[u8; 512]>::new();
    let mut size = 0;
    loop {
        /* Carve out 512 bytes at a time */
        let mut buf: [u8; 512] = [0; 512];
        let len = file.read(&mut buf).expect("Failed to read");

        /* If read len == 0, we've hit the EOF */
        if len == 0 {
            break;
        }

        /* Save this chunk */
        out.push(buf);
        size += len;

        /* If we've hit the requested file size, end now */
        if size >= file_size {
            break;
        }
    }
    out
}

fn checksum_header(tar: tar_header) -> u64 {
    let mut node = Vec::<tar_node>::new();
    node.push(tar_node {
        header: tar,
        data: Vec::<[u8; 512]>::new()
    });
    let out = serialize(&node);
    let mut checksum: u64 = 0;
    for i in &out[..512] {
        checksum += *i as u64;
    }
    checksum
}

fn serialize(tar: &Vec<tar_node>) -> Vec<u8> {
    /* Serialize the header and data for writing */
    let mut out = Vec::<u8>::new();
    /* Iterate through each header value */
    for node in tar {
        out.extend_from_slice(&node.header.file_name);
        out.extend_from_slice(&node.header.file_mode);
        out.extend_from_slice(&node.header.own_user);
        out.extend_from_slice(&node.header.own_group);
        out.extend_from_slice(&node.header.file_size);
        out.extend_from_slice(&node.header.mod_time);
        out.extend_from_slice(&node.header.header_checksum);
        out.extend_from_slice(&node.header.link_indicator);
        out.extend_from_slice(&node.header.link_name);
        out.extend_from_slice(&node.header.ustar_magic);
        out.extend_from_slice(&node.header.ustar_version);
        out.extend_from_slice(&node.header.own_user_name);
        out.extend_from_slice(&node.header.own_group_name);
        out.extend_from_slice(&node.header.device_major);
        out.extend_from_slice(&node.header.device_minor);
        out.extend_from_slice(&node.header.file_prefix);
        out.extend_from_slice(&node.header.reserved);
        /* Iterate through each data chunk */
        for d in &node.data {
            out.extend_from_slice(d);
        }
    }
    out
}

fn append_end(tar: &mut Vec<tar_node>) {
    /* Append the empty blocks of 0x00's at the end */
    let mut node = tar_node::default();
    let mut i = 0;
    loop {
        node.data.push([0; 512]);
        i += 1;
        if i > 16 {
            break;
        }
    }
    tar.push(node);
}

fn oct_to_dec(input: &[u8]) -> usize {
    /* Convert the &[u8] to string and remove the null byte */
    let mut s = str::from_utf8(&input).expect("Cannot convert utf8").to_string();
    s.pop();

    /* Convert to usize from octal */
    usize::from_str_radix(&s, 8).expect("Cannot convert oct to decimal")
}

//Incomplete
fn dec_to_oct(input: usize) -> Vec<u8> {
    Vec::<u8>::new()
}
