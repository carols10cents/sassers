use std::result;
use std::io;

pub type Result<T> = result::Result<T, SassError>;

#[derive(Debug,PartialEq)]
pub struct SassError {
    pub message: String,
    pub kind: ErrorKind,
}

impl From<io::Error> for SassError {
    fn from(err: io::Error) -> SassError {
        SassError { message: format!("IO error! {}", err), kind: ErrorKind::IoError }
    }
}

#[derive(Debug,PartialEq)]
pub enum ErrorKind {
    IoError,
    InvalidOutputStyle,
    InvalidColor,
    TokenizerError,
    ExpectedValue,
    ExpectedOperator,
    ExpectedMixin,
    ExpectedMixinArgument,
    UnexpectedEof,
    UnexpectedTopLevelElement,
    InvalidOperator,
    InvalidApplyListArgs,
    InvalidApplyMathArgs,
    InvalidSquareUnits,
    IncompatibleUnits,
    UnknownFunction,
    ArgumentNotFound,
    ParseError,
    UnexpectedValuePartType,
}
