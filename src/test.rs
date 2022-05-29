#[allow(unused_imports)]
use crate::tar::*;
use std::fs::File;

#[test]
fn open_tar_file() {
    TarFile::open("test/1.tar".to_string()).unwrap();
}

#[test]
fn new_and_write_tar_file() {
    let data = TarFile::new("test/1.txt".to_string()).unwrap();

    let out = File::create("test/2.tar".to_string()).unwrap();
    data.write(&out).unwrap();
}

#[test]
fn new_and_append_tar_file() {
    let mut data = TarFile::new("test/1.txt".to_string()).unwrap();
    data.append("test/1.txt".to_string()).unwrap();

    let out = File::create("test/5.tar".to_string()).unwrap();
    data.write(&out).unwrap();
}

#[test]
fn open_and_write_tar_file() {
    let data = TarFile::open("test/1.tar".to_string()).unwrap();

    let out = File::create("test/5.tar".to_string()).unwrap();
    data.write(&out).unwrap();
}
