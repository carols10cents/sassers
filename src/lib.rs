#[macro_use]
extern crate log;

extern crate regex;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex;
use std::io::Write;

mod ast;
mod context;
mod error;
mod sass;
mod operator_or_token;
mod optimizer;
mod parser;
mod token;
mod tokenizer;

use context::Context;
use error::Result;
use tokenizer::Tokenizer;
use parser::Parser;
use sass::output_style::SassOutputStyle;

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

    let style: SassOutputStyle = try!(style.parse());

    match style {
        SassOutputStyle::Tokens => {
            let mut tokenizer = Tokenizer::new(&imports_resolved);
            while let Some(token) = tokenizer.next() {
                try!(write!(output, "{:?}\n", token))
            }
        },
        SassOutputStyle::AST => {
            let mut parser = Parser::new(&imports_resolved);
            while let Some(ast_root) = parser.next() {
                try!(write!(output, "{:#?}\n", ast_root));
            }
        },
        other_style => {
            let mut parser  = Parser::new(&imports_resolved);
            let mut context = Context::new();
            while let Some(Ok(ast_root)) = parser.next() {
                let evaluated = ast_root.evaluate(&mut context);
                if let Some(root) = evaluated {
                    let optimized = optimizer::optimize(root);
                    for r in optimized {
                        try!(r.stream(output, other_style));
                    }
                }
            }
        },
    }

    Ok(())
}
