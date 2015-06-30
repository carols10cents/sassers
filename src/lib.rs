extern crate regex;
use std::path::Path;
use std::fs::File;
use std::io::Read;

mod event;
mod output;
mod sass;
mod tokenizer;
mod top_level_event;
mod variable_mapper;

use tokenizer::Tokenizer;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub kind: ErrorKind,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error { message: format!("IO error! {}", err), kind: ErrorKind::IoError }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    IoError,
    InvalidStyle,
}

pub fn compile(inputfile: &str, style: &str) -> Result<String> {
    let mut file = try!(File::open(&Path::new(inputfile)));
    let mut sass = String::new();

    try!(file.read_to_string(&mut sass));
    let mut st = Tokenizer::new(&sass);
    match style {
        "nested"     => Ok(output::nested(&mut st)),
        "compressed" => Ok(output::compressed(&mut st)),
        "expanded"   => Ok(output::expanded(&mut st)),
        "compact"    => Ok(output::compact(&mut st)),
        "debug"      => Ok(output::debug(&mut st)),
        _            => {
            Err(Error {
                kind: ErrorKind::InvalidStyle,
                message: format!("Unknown style {:?}. Please specify one of nested, compressed, expanded, or compact.", style),
            })
        },
    }
}
