use tokenizer::Tokenizer;
use error::{Result};

pub struct Parser<'a> {
    pub text: &'a str,
    pub tokenizer: Tokenizer<'a>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_none_for_empty_string() {
        let mut parser = Parser::new("");
        assert_eq!(parser.next(), None);
    }
}
