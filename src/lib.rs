use std::fs::File;
use std::str;
use std::io::Write;

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
        }
    }
}

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

pub fn file_open(filename: String) -> Vec<tar_node> {
    let file = File::open(filename).expect("Could not open file");

    let out = ingest(file);

    out
}

pub fn tar_write(filename: String, tar: &mut Vec<tar_node>) {
    append_end(tar);
    let flat = serialize(&tar);

    let mut file = File::create(filename).expect("Error creating file");

    file.write_all(&flat);
    file.flush();
}

fn ingest(filename: File) -> Vec<tar_node> {
    let mut tar = Vec::<tar_node>::new();
    match generate_header(&filename) {
        Some(n) => {
            tar.push(tar_node {
                header: n,
                data: parse_data(&filename),
            });
        }
        _ => {}
    };
    tar
}

//Incomplete
fn generate_header(filename: &File) -> Option<tar_header> {
    let header: tar_header = tar_header::default();
    Some(header)
}

fn parse_data<T: std::io::Read>(mut file: T) -> Vec<[u8; 512]> {
    let mut out = Vec::<[u8; 512]>::new();
    loop {
        let mut buf: [u8; 512] = [0; 512];
        let len = file.read(&mut buf).expect("Failed to read");
        if len == 0 {
            break;
        }

        out.push(buf)
    }
    out
}

fn serialize(tar: &Vec<tar_node>) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
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
        for d in &node.data {
            out.extend_from_slice(d);
        }
    }
    out
}

fn append_end(tar: &mut Vec<tar_node>) {
    let mut node = tar_node::default();
    node.data.push([0; 512]);
    tar.push(node);
}

//Incomplete
fn convert_header_to_dec(header: tar_header) -> tar_header {
    let header: tar_header = tar_header::default();
    header
}

//Incomplete
fn convert_header_to_oct(header: tar_header) -> tar_header {
    let header: tar_header = tar_header::default();
    header
}

fn oct_to_dec(input: &[u8]) -> usize {
    usize::from_str_radix(str::from_utf8(&input).expect("Cannot convert utf8"), 8).expect("Cannot convert oct to decimal")
}

//Incomplete
fn dec_to_oct(input: usize) -> Vec<u8> {
    Vec::<u8>::new()
}
