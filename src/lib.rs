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
    file_name: [char; 100],
    file_mode: [char; 8],
    own_user: [char; 8],
    own_group: [char; 8],
    file_size: [char; 12],
    mod_time: [char; 12],
    header_checksum: [char; 8],
    link_indicator: [char; 1],
    link_name: [char; 100],
    ustar_magic: [char; 6],
    ustar_version: [char; 2],
    own_user_name: [char; 32],
    own_group_name: [char; 32],
    device_major: [char; 8],
    device_minor: [char; 8],
    file_prefix: [char; 155],
}

pub struct tar_node {
    header: tar_header,
    data: Vec<u8>,
}

pub fn tar_append(filename: File, tar: Option<tar_header>) {}

pub fn tar_open(filename: File) {}

pub fn tar_close(tar: Vec<tar_node>) {}

pub fn tar_write(tar: Vec<tar_node>) {}
