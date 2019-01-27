#[macro_use]
extern crate log;

extern crate regex;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

mod ast;
mod context;
mod error;
mod expression_evaluator;
mod operator;
mod operator_offset;
mod operator_or_token;
mod optimizer;
mod parser;
mod sass;
mod token;
mod token_offset;
mod tokenizer;

use crate::context::Context;
use crate::error::Result;
use crate::parser::Parser;
use crate::sass::output_style::{Compact, Compressed, Debug, Expanded, Nested, SassOutputStyle};
use crate::tokenizer::Tokenizer;

fn resolve_imports(inputpath: &PathBuf) -> Result<String> {
    let mut file = File::open(&inputpath)?;
    let mut sass = String::new();

    file.read_to_string(&mut sass)?;

    let mut imports_resolved = String::new();
    let re = Regex::new("@import \"([^\"]*)\";").unwrap();
    for line in sass.split("\n") {
        match re.captures(line) {
            Some(caps) => {
                let imported = resolve_imports(
                    &inputpath.with_file_name(caps.get(1).unwrap().as_str())
                )?;
                imports_resolved.push_str(&imported);
            }
            None => {
                imports_resolved.push_str(line);
            }
        }
        imports_resolved.push_str("\n");
    }
    Ok(imports_resolved)
}

pub fn compile(input_filename: &str, output: &mut Write, style: &str) -> Result<()> {
    let input_path = PathBuf::from(input_filename);
    let imports_resolved = resolve_imports(&input_path)?;

    match style {
        "tokens" => {
            let mut tokenizer = Tokenizer::new(&imports_resolved);
            while let Some(token) = tokenizer.next() {
                writeln!(output, "{:?}", token)?;
            }
        }
        "ast" => {
            let mut parser = Parser::new(&imports_resolved);
            while let Some(root) = parser.next() {
                writeln!(output, "{:#?}", root)?;
            }
        }
        other => {
            let style: Box<SassOutputStyle> = get_style(other);
            let mut parser = Parser::new(&imports_resolved);
            let mut context = Context::new();
            while let Some(Ok(ast_root)) = parser.next() {
                let evaluated = ast_root.evaluate(&mut context);
                if let Some(root) = evaluated {
                    let optimized = optimizer::optimize(root);
                    for r in optimized.into_iter() {
                        r.stream(output, &*style)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn get_style(style: &str) -> Box<SassOutputStyle> {
    match style {
        "nested"     => Box::new(Nested {}),
        "compressed" => Box::new(Compressed {}),
        "expanded"   => Box::new(Expanded {}),
        "compact"    => Box::new(Compact {}),
        "debug"      => Box::new(Debug {}),
        style        => panic!("Unknown output style {:?}. Please specify one of nested, compressed, expanded, or compact.", style),
    }
}
