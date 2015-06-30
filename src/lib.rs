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

pub fn compile(inputfile: &str, style: &str) -> Result<String, &'static str> {
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
                _            => Err("Unknown style:. Please specify one of nested, compressed, expanded, or compact."),
            }
        },
        Err(msg) => panic!("Could not read file! {}", msg),
    }
}

