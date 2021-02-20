use crate::tar::*;

#[test]
fn open_tar_file() {
    let data = file_open("1.tar".to_string());

    println!("{}", data.len());
}

#[test]
fn write_tar_file() {
    let mut data = file_open("1.tar".to_string());

    tar_write("2.tar".to_string(), &mut data);
}
