use tokenizer::Tokenizer;
use sass::output_style::SassOutputStyle;
use token::{Lexeme, Token};
use error::{Result, SassError, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    List(Vec<Lexeme>),
}

impl Expression {
    pub fn to_string(&self, style: SassOutputStyle) -> String {
        match *self {
            Expression::List(ref elements) => elements.iter().map(|e|
                e.token.to_string()
            ).collect::<Vec<_>>().join(" "),
        }
    }

    pub fn parse(tokenizer: &mut Tokenizer) -> Result<Expression> {
        let mut list = vec![];
        while let Some(Ok(lexeme)) = tokenizer.next() {
            if lexeme.token == Token::Semicolon {
                return Ok(Expression::List(list))
            } else {
                list.push(lexeme);
            }
        }

        let error_offset = match list.pop() {
            Some(lexeme) => lexeme.offset.unwrap_or(0),
            None => 0,
        };
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected semicolon while parsing a value expression; reached EOF instead."
            ),
        })
    }
}