#[allow(unused_imports)]
use crate::tar::*;

#[test]
fn open_tar_file() {
    file_read("test/1.tar".to_string());
}

#[test]
fn write_tar_file() {
    let mut data = file_read("test/1.txt".to_string());

    tar_write("test/2.tar".to_string(), &mut data);
}

#[test]
fn read_write_tar_file() {
    let mut data = tar_read("test/1.tar".to_string());

    tar_write("test/3.tar".to_string(), &mut data);
}
