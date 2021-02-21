use crate::tar::*;

#[test]
fn open_tar_file() {
    let data = file_read("1.tar".to_string());

    println!("{}", data.len());
}

#[test]
fn write_tar_file() {
    let mut data = file_read("1.txt".to_string());

    tar_write("2.tar".to_string(), &mut data);
}

#[test]
fn read_write_tar_file() {
    let mut data = tar_read("1.tar".to_string());

    tar_write("3.tar".to_string(), &mut data);
}
