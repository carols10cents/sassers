#![feature(collections)]

#[macro_use]
extern crate log;

extern crate regex;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex;

mod error;
mod evaluator;
mod event;
mod inner_tokenizer;
mod sass;
mod substituter;
mod tokenizer;
mod tokenizer_utils;
mod value_tokenizer;

use error::Result;
use tokenizer::Tokenizer;

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
    Ok(try!(tokenizer.output(try!(style.parse()))))
}
