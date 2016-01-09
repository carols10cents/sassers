#[macro_use]
extern crate log;

extern crate regex;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex;
use std::io::Write;
use std::iter;

mod error;
mod evaluator;
mod event;
mod inner_tokenizer;
mod sass;
mod substituter;
mod token;
mod tokenizer;
mod tokenizer_utils;
mod value_tokenizer;

use error::{SassError, Result};
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

pub fn compile<W: Write>(input_filename: &str, output: &mut W, style: &str) -> Result<()> {
    let input_path = PathBuf::from(input_filename);
    let imports_resolved = try!(resolve_imports(&input_path));
    let max_offset = imports_resolved.len();

    let mut tokenizer = Tokenizer::new(&imports_resolved);
    let style = try!(style.parse());

    match tokenizer.stream(output, style) {
        Err(sass_error) => {
            let before_error = &imports_resolved[0..sass_error.offset];
            let after_error = &imports_resolved[sass_error.offset..];

            let context_start = before_error
                .as_bytes()
                .iter()
                .rposition(|&c| c == b'\n')
                .unwrap_or(0);
            let context_end = after_error
                .as_bytes()
                .iter()
                .position(|&c| c == b'\n')
                .unwrap_or(max_offset);

            Err(SassError {
                message: format!("{}\nOn line {}:\n{}\n{}^",
                    sass_error.message,
                    before_error.split("\n").collect::<Vec<_>>().len(),
                    &imports_resolved[context_start..context_end + sass_error.offset],
                    iter::repeat(" ")
                         .take(sass_error.offset - context_start)
                         .collect::<Vec<_>>()
                         .join(""),
                ),
                ..sass_error
            })
        },
        other => other,
    }
}
