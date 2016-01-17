use tokenizer::Tokenizer;
use error::{Result};

pub struct Parser<'a> {
    pub text: &'a str,
    pub tokenizer: Tokenizer<'a>,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<ASTNode>;

    fn next(&mut self) -> Option<Result<ASTNode>> {
        None
    }
}

impl<'a> Parser<'a> {
    pub fn new(text: &str) -> Parser {
        Parser {
            text: text,
            tokenizer: Tokenizer::new(&text),
        }
    }
}
