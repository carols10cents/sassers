use std::error::{self, Error};
use std::fmt;
use std::io;
use std::result;

pub type Result<T> = result::Result<T, SassError>;

#[derive(Debug, PartialEq)]
pub struct SassError {
    pub message: String,
    pub kind: ErrorKind,
    pub offset: usize,
}

impl fmt::Display for SassError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} at {}: {}", self.kind, self.offset, self.message)
    }
}

impl error::Error for SassError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

impl From<io::Error> for SassError {
    fn from(err: io::Error) -> SassError {
        SassError {
            offset: 0,
            message: String::from(err.description()),
            kind: ErrorKind::IoError,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    IoError,
    InvalidOutputStyle,
    TokenizerError,
    ParserError,
    UnexpectedEof,
}
