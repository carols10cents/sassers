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

    pub fn parse<T: Iterator<Item = Result<Lexeme>>>(tokenizer: &mut T) -> Result<Expression> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use token::{Lexeme, Token};

    fn semicolon() -> Lexeme {
        Lexeme { token: Token::Semicolon, offset: None }
    }

    fn blue() -> Lexeme {
        Lexeme { token: Token::Ident("blue".into()), offset: None }
    }

    #[test]
    fn it_parses_a_list() {
        let mut fake_tokenizer = vec![Ok(blue()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::List(vec![blue()]))
        );
    }
}