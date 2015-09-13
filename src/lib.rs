#![feature(append)]
#![feature(collections)]

#[macro_use]
extern crate log;

extern crate regex;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex;

mod evaluator;
mod event;
mod output;
mod sass;
mod tokenizer;
mod top_level_event;
mod value_tokenizer;
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

fn resolve_imports(inputpath: &PathBuf) -> Result<String> {
    let mut file = try!(File::open(&inputpath));
    let mut sass = String::new();

    try!(file.read_to_string(&mut sass));

    let mut imports_resolved = String::new();
    for line in sass.split("\n") {
        let re = Regex::new("@import \"([^\"]*)\";").unwrap();

        match re.captures(line) {
            Some(caps) => {
                let imported = try!(resolve_imports(&inputpath.with_file_name(caps.at(1).unwrap())));
                imports_resolved.push_str(&imported);
            },
            None => {
                imports_resolved.push_str(line);
            },
        }
        imports_resolved.push_str("\n");
    }
    Ok(imports_resolved)
}

pub fn compile(inputfile: &str, style: &str) -> Result<String> {
    let inputpath = PathBuf::from(inputfile);
    let imports_resolved = try!(resolve_imports(&inputpath));

    let mut tokenizer = Tokenizer::new(&imports_resolved);
    match style {
        "nested"     => Ok(output::nested(&mut tokenizer)),
        "compressed" => Ok(output::compressed(&mut tokenizer)),
        "expanded"   => Ok(output::expanded(&mut tokenizer)),
        "compact"    => Ok(output::compact(&mut tokenizer)),
        "debug"      => Ok(output::debug(&mut tokenizer)),
        _            => {
            Err(Error {
                kind: ErrorKind::InvalidStyle,
                message: format!("Unknown style {:?}. Please specify one of nested, compressed, expanded, or compact.", style),
            })
        },
    }
}
