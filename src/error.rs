use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TarError {
    #[error("DekuError: {0}")]
    Deku(#[from] deku::DekuError),
    #[error("IoError: {0}")]
    Io(#[from] io::Error),
    #[error("Error in conversion of oct_to_dev")]
    Utf8Error(#[from] Utf8Error),
    #[error("Error in conversion of oct_to_dev")]
    ParseIntError(#[from] ParseIntError),
    #[error("End of tar")]
    EndOfTar,
    #[error("Invalid magic")]
    InvalidMagic,
    #[error("Invalid Checksum")]
    InvalidChecksum,
}
