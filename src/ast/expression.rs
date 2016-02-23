use sass::output_style::SassOutputStyle;
use token::{Lexeme, Token};
use ast::number_value::NumberValue;
use context::Context;
use error::{Result, SassError, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    List(Vec<Expression>),
    Number(NumberValue),
    Operator(Lexeme),
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
            Expression::Operator(ref lex) => lex.token.to_string(),
            Expression::String(ref lex) => lex.token.to_string(),
        }
    }

    pub fn parse<T: Iterator<Item = Result<Lexeme>>>(tokenizer: &mut T) -> Result<Expression> {
        let mut list = vec![];
        while let Some(Ok(lexeme)) = tokenizer.next() {
            match lexeme.token {
                Token::Semicolon => {
                    if list.len() == 1 {
                        return Ok(list.pop().unwrap())
                    } else {
                        return Ok(Expression::List(list))
                    }
                },
                Token::Number(_, _) => {
                    list.push(Expression::Number(NumberValue::from_scalar(lexeme)));
                },
                Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent => {
                    list.push(Expression::Operator(lexeme));
                },
                _ => {
                    list.push(Expression::String(lexeme));
                }
            }
        }

        let error_offset = match list.pop() {
            Some(Expression::String(lexeme)) => lexeme.offset.unwrap_or(0),
            Some(Expression::Operator(lexeme)) => lexeme.offset.unwrap_or(0),
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

    pub fn evaluate(self, context: &Context) -> Expression {
        match self {
            Expression::Number(nv) => Expression::Number(nv),
            Expression::Operator(op) => Expression::Operator(op),
            Expression::String(lex) => {
                context.get_variable(&lex).unwrap_or(Expression::String(lex))
            },
            Expression::List(exprs) => {
                let mut last_was_an_operator = true;
                // TODO: I'd love to have more types here, to ensure only values are
                // on the value stack and only operators are on the operator stack...
                let mut value_stack: Vec<Expression> = Vec::new();
                let mut op_stack: Vec<Expression> = Vec::new();

                // Split into value stacks and operator stacks

                let mut exprs = exprs.into_iter();
                while let Some(part) = exprs.next() {
                    match part {
                        Expression::Number(nv) => {
                            if last_was_an_operator {
                                value_stack.push(Expression::Number(nv));
                            }
                            last_was_an_operator = false;
                        },
                        Expression::Operator(op) => {
                            op_stack.push(Expression::Operator(op));
                            last_was_an_operator = true;
                        },
                        Expression::String(lex) => {
                            value_stack.push(Expression::String(lex));
                            last_was_an_operator = false;
                        },
                        Expression::List(list) => {
                            value_stack.push(Expression::List(list));
                            last_was_an_operator = false;
                        },
                    }
                }

                // Process the stacks
                while !op_stack.is_empty() {
                    let op = op_stack.pop();
                    let second = value_stack.pop();
                    let first = value_stack.pop();
                    value_stack.push(apply_math(first, op, second));
                }

                value_stack.pop().unwrap()
            },
        }
    }
}

fn apply_math(first: Option<Expression>, operator: Option<Expression>, second: Option<Expression>) -> Expression {
    let first = first.unwrap();
    let operator = operator.unwrap();
    let second = second.unwrap();

    match (first.clone(), second) {
        (Expression::Number(f), Expression::Number(s)) => {
            match operator {
                Expression::Operator(o) => {
                    let result = f.apply_math(&o, &s);
                    Expression::Number(result)
                },
                _ => unreachable!(),
            }
        },
        _ => first,
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
        Lexeme { token: Token::String("blue".into()), offset: None }
    }

    fn plus() -> Lexeme {
        Lexeme { token: Token::Plus, offset: None }
    }

    fn one() -> Lexeme {
        Lexeme { token: Token::Number(1.0, None), offset: None }
    }

    fn one_px() -> Lexeme {
        Lexeme { token: Token::Number(1.0, Some("px".into())), offset: None }
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
        let mut fake_tokenizer = vec![
            Ok(one()),
            Ok(plus()),
            Ok(one()),
            Ok(semicolon())
        ].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::List(vec![
                Expression::Number(NumberValue::from_scalar(one())),
                Expression::Operator(plus()),
                Expression::Number(NumberValue::from_scalar(one())),
            ]))
        );
    }

    #[test]
    fn it_parses_a_number_without_units() {
        let mut fake_tokenizer = vec![Ok(one()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Number(NumberValue::from_scalar(one())))
        );
    }

    #[test]
    fn it_parses_a_number_with_units() {
        let mut fake_tokenizer = vec![Ok(one_px()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Number(NumberValue::from_scalar(one_px())))
        );
    }
}