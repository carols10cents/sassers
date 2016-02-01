use std::result;
use std::io;

pub type Result<T> = result::Result<T, SassError>;

#[derive(Debug,PartialEq)]
pub struct SassError {
    pub message: String,
    pub kind: ErrorKind,
    pub offset: usize,
}

impl From<io::Error> for SassError {
    fn from(err: io::Error) -> SassError {
        SassError {
            offset: 0,
            message: format!("IO error! {}", err),
            kind: ErrorKind::IoError
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum ErrorKind {
    IoError,
    InvalidOutputStyle,
    TokenizerError,
    UnexpectedEof,
}
