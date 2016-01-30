use sass::output_style::SassOutputStyle;
use token::{Lexeme, Token};
use ast::number_value::NumberValue;
use error::{Result, SassError, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    List(Vec<Expression>),
    Number(NumberValue),
    String(Lexeme),
}

impl Expression {
    #[allow(unused_variables)]
    pub fn to_string(&self, style: SassOutputStyle) -> String {
        match *self {
            Expression::List(ref elements) => {
                elements.iter().map(|e|
                    e.to_string(style)
                ).collect::<Vec<_>>().join(" ")
            },
            Expression::Number(ref nv) => nv.to_string(),
            Expression::String(ref lex) => lex.token.to_string(),
        }
    }

    pub fn parse<T: Iterator<Item = Result<Lexeme>>>(tokenizer: &mut T) -> Result<Expression> {
        let mut list = vec![];
        while let Some(Ok(lexeme)) = tokenizer.next() {
            if lexeme.token == Token::Semicolon {
                if list.len() == 1 {
                    return Ok(list.pop().unwrap())
                } else {
                    return Ok(Expression::List(list))
                }
            } else if let Token::Number(_) = lexeme.token {
                list.push(Expression::Number(NumberValue::from_scalar(lexeme)))
            } else {
                list.push(Expression::String(lexeme));
            }
        }

        let error_offset = match list.pop() {
            Some(Expression::String(lexeme)) => lexeme.offset.unwrap_or(0),
            Some(Expression::Number(nv)) => nv.offset().unwrap_or(0),
            Some(Expression::List(_)) => unreachable!(), // for now until nested lists
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
    use ast::number_value::NumberValue;

    fn semicolon() -> Lexeme {
        Lexeme { token: Token::Semicolon, offset: None }
    }

    fn blue() -> Lexeme {
        Lexeme { token: Token::Ident("blue".into()), offset: None }
    }

    fn zero() -> Lexeme {
        Lexeme { token: Token::Number(0.0), offset: None }
    }

    #[test]
    fn it_parses_a_single_string() {
        let mut fake_tokenizer = vec![Ok(blue()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::String(blue()))
        );
    }

    #[test]
    fn it_parses_a_list() {
        let mut fake_tokenizer = vec![Ok(zero()), Ok(zero()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::List(vec![
                Expression::Number(NumberValue::from_scalar(zero())),
                Expression::Number(NumberValue::from_scalar(zero())),
            ]))
        );
    }

    #[test]
    fn it_parses_a_number_without_units() {
        let mut fake_tokenizer = vec![Ok(zero()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Number(NumberValue::from_scalar(zero())))
        );
    }

    #[test]
    fn it_parses_a_number_with_units() {

    }
}