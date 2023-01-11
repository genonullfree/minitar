use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Debug)]
pub enum LibraryError {
    Filetype(FileTypeError),
    Io(io::Error),
    Tar(TarError),
}

impl From<FileTypeError> for LibraryError {
    fn from(error: FileTypeError) -> Self {
        LibraryError::Filetype(error)
    }
}

impl From<TarError> for LibraryError {
    fn from(error: TarError) -> Self {
        LibraryError::Tar(error)
    }
}

impl From<io::Error> for LibraryError {
    fn from(error: io::Error) -> Self {
        LibraryError::Io(error)
    }
}

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

#[derive(Debug)]
pub enum FileTypeError {
    Unknown,
    IoError,
    InvalidMetaData,
}
