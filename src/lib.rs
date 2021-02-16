use std::fs::File;

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
    data: [u8; 512],
}

impl Default for tar_node {
    fn default() -> tar_node {
        tar_node {
            header: tar_header::default(),
            data: [0; 512],
        }
    }
}

pub fn tar_append(filename: File, tar: Vec<tar_node>) -> Vec<tar_node> {
    Vec::<tar_node>::new()
}

pub fn tar_open(filename: File) -> Vec<tar_node> {
    Vec::<tar_node>::new()
}

pub fn tar_close(tar: Vec<tar_node>) {}

pub fn tar_write(tar: Vec<tar_node>) {}

fn ingest(filename: File) -> Vec<tar_node> {
    Vec::<tar_node>::new()
}

fn parse_header(filename: File) -> Option<tar_header> {
    let header: tar_header = tar_header::default();
    Some(header)
}

fn parse_data(filename: File) {}

fn serialize(tar: Vec<tar_node>) -> Vec<u8> {
    Vec::<u8>::new()
}

fn append_empty(tar: &mut Vec<tar_node>) {
    tar.push(tar_node::default());
}

fn convert_header_to_dec(header: tar_header) -> tar_header {
    let header: tar_header = tar_header::default();
    header
}

fn convert_header_to_oct(header: tar_header) -> tar_header {
    let header: tar_header = tar_header::default();
    header
}

fn oct_to_dec(input: Vec<char>) -> usize {
    0
}

fn dec_to_oct(input: usize) -> Vec<char> {
    Vec::<char>::new()
}
