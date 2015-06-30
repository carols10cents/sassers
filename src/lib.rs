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

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    FileNotFound,
    FileNotReadable,
    InvalidStyle,
}

pub fn compile(inputfile: &str, style: &str) -> Result<String> {
    let mut file = match File::open(&Path::new(inputfile)) {
        Ok(f) => f,
        Err(msg) => panic!("File not found! {}", msg),
    };
    let mut sass = String::new();
    match file.read_to_string(&mut sass) {
        Ok(_) => {
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
        },
        Err(msg) => panic!("Could not read file! {}", msg),
    }
}
