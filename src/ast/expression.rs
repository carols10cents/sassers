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

impl From<Lexeme> for Expression {
    fn from(lexeme: Lexeme) -> Expression {
        match lexeme.token {
            Token::Number(..) =>
                Expression::Number(
                    NumberValue::from_scalar(lexeme)
                ),
            Token::Plus | Token::Minus | Token::Star |
                          Token::Slash | Token::Percent =>
                Expression::Operator(lexeme),
            _ => Expression::String(lexeme),
        }
    }
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

    fn parse_parenthetical<T: Iterator<Item = Result<Lexeme>>>(tokenizer: &mut T) -> Result<Expression> {
        let mut list = vec![];
        while let Some(Ok(lexeme)) = tokenizer.next() {
            match lexeme.token {
                Token::RightParen => {
                    return Ok(Expression::List(list))
                },
                Token::LeftParen => {
                    list.push(
                        try!(Expression::parse_parenthetical(tokenizer))
                    );
                },
                Token::Comment(_) => {},
                _ => list.push(lexeme.into()),
            }
        }

        let error_offset = match list.pop() {
            Some(Expression::String(lexeme)) => lexeme.offset.unwrap_or(0),
            Some(Expression::Operator(lexeme)) => lexeme.offset.unwrap_or(0),
            Some(Expression::Number(nv)) => nv.offset.unwrap_or(0),
            Some(Expression::List(_)) => unreachable!(), // for now until nested lists
            None => 0,
        };
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected right paren while parsing a value expression; reached EOF instead."
            ),
        })
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
                Token::LeftParen => {
                    list.push(
                        try!(Expression::parse_parenthetical(tokenizer))
                    );
                },
                Token::Comment(_) => {},
                _ => list.push(lexeme.into()),
            }
        }

        let error_offset = match list.pop() {
            Some(Expression::String(lexeme)) => lexeme.offset.unwrap_or(0),
            Some(Expression::Operator(lexeme)) => lexeme.offset.unwrap_or(0),
            Some(Expression::Number(nv)) => nv.offset.unwrap_or(0),
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

    fn evaluate_list(exprs: Vec<Expression>, context: &Context, force_slash: bool) -> Expression {

        let mut last_was_an_operator = true;
        // TODO: I'd love to have more types here, to ensure only values are
        // on the value stack and only operators are on the operator stack...
        let mut value_stack: Vec<Expression> = Vec::new();
        let mut op_stack: Vec<Expression> = Vec::new();

        // Split into value stacks and operator stacks

        let mut exprs = exprs.into_iter();
        while let Some(part) = exprs.next() {
            debug!("Processing list item {:#?}", part);
            match part {
                Expression::Number(nv) => {
                    if last_was_an_operator {
                        debug!("Push on value stack {:#?}", nv);
                        value_stack.push(Expression::Number(nv));
                    } else {
                        let list = create_list(
                            value_stack.pop(),
                            Expression::Number(nv),
                        );
                        debug!("Push on list on value stack {:#?}", list);
                        value_stack.push(list);
                    }
                    last_was_an_operator = false;
                },
                Expression::Operator(Lexeme {
                    token: Token::Slash, offset: o }) if !force_slash => {
                    let list = create_list(
                        value_stack.pop(),
                        Expression::Operator(Lexeme {
                            token: Token::Slash, offset: o
                        }),
                    );
                    debug!("Push on list on value stack {:#?}", list);
                    value_stack.push(list);
                },
                Expression::Operator(op) => {
                    debug!("Push on op stack {:#?}", op);
                    op_stack.push(Expression::Operator(op));
                    last_was_an_operator = true;
                },
                Expression::String(lex) => {
                    let var_eval = context.get_variable(&lex)
                                    .unwrap_or(Expression::String(lex));

                    if last_was_an_operator {
                        value_stack.push(var_eval);
                    } else {
                        let list = create_list(
                            value_stack.pop(),
                            var_eval,
                        );
                        value_stack.push(list);
                    }
                    last_was_an_operator = false;
                },
                Expression::List(list) => {
                    debug!("Push list on value stack {:#?}", list);
                    value_stack.push(Expression::List(list));
                    last_was_an_operator = false;
                },
            }
        }

        // Process the stacks
        while !op_stack.is_empty() {
            let op = op_stack.pop().unwrap();
            let second = value_stack.pop().expect("Expected a second argument on the value stack");
            let first = value_stack.pop().expect("Expected a first argument on the value stack");

            let math_result = first.apply_math(op, second, context, force_slash);
            debug!("Math result: {:#?}", math_result);

            value_stack.push(math_result);
        }

        value_stack.pop().unwrap()
    }

    pub fn evaluate(self, context: &Context) -> Expression {
        match self {
            Expression::Number(nv) => Expression::Number(nv),
            Expression::Operator(op) => Expression::Operator(op),
            Expression::String(lex) => {
                context.get_variable(&lex).unwrap_or(Expression::String(lex))
            },
            Expression::List(exprs) => {
                Expression::evaluate_list(exprs, context, false)
            },
        }
    }

    fn is_slash(&self) -> bool {
        match self {
            &Expression::Operator(ref o) if o.token == Token::Slash => true,
            _ => false,
        }
    }

    fn is_plus(&self) -> bool {
        match self {
            &Expression::Operator(ref o) if o.token == Token::Plus => true,
            _ => false,
        }
    }

    fn apply_math(self, operator: Expression, second: Expression, context: &Context, force_slash: bool) -> Expression {
        debug!("Applying math to:\nfirst: {:#?}\nop: {:#?}\nsecond: {:#?}", self, operator, second);
        match (self.clone(), second.clone()) {
            (Expression::Number(f), Expression::Number(s)) => {
                match operator {
                    Expression::Operator(Lexeme {
                        token: Token::Slash, offset: o }) => {
                        if force_slash || (f.computed || s.computed) {
                            let result = f.apply_math(
                                &Lexeme {
                                    token: Token::Slash,
                                    offset: o
                                },
                                &s
                            );
                            Expression::Number(result)
                        } else {
                            Expression::List(vec![self, operator.clone(), second.clone()])
                        }
                    },
                    Expression::Operator(o) => {
                        let result = f.apply_math(&o, &s);
                        Expression::Number(result)
                    },
                    _ => unreachable!(),
                }
            },
            (Expression::List(f), Expression::List(s)) => {
                let eval_first = Expression::evaluate_list(f, context, true);
                let eval_second = Expression::evaluate_list(s, context, true);
                eval_first.apply_math(operator, eval_second, context, true)
            },
            (Expression::List(f), Expression::Number(s)) => {
                let eval_first = Expression::evaluate_list(f, context, true);
                eval_first.apply_math(operator, Expression::Number(s), context, true)
            },
            (Expression::Number(f), Expression::List(s)) => {
                if s.iter().any(|e| e.is_slash()) {
                    let evaled_list = Expression::evaluate_list(s, context, true);
                    Expression::Number(f).apply_math(
                        operator,
                        evaled_list,
                        context,
                        true
                    )
                } else {
                    if let Some((ref first_in_list, rest_of_list)) =
                            s.split_first() {
                        let new_first = if operator.is_plus() {
                            Expression::String(
                                Lexeme {
                                    token: Token::String(
                                        format!("{}{}", f, first_in_list.to_string(SassOutputStyle::Expanded))
                                    ),
                                    offset: f.offset,
                                }
                            )
                        } else {
                            Expression::Number(f).apply_math(
                                operator,
                                (*first_in_list).clone(),
                                context,
                                true
                            )
                        };

                        let mut new_list = vec![new_first];
                        new_list.extend_from_slice(rest_of_list);
                        Expression::List(new_list)
                    } else {
                        panic!("Trying to perform an operation on a number and a list; expected to get a list with something in it");
                    }
                }
            },
            _ => unimplemented!(),
        }
    }
}

fn create_list(head: Option<Expression>, tail: Expression) -> Expression {
    let mut list = match head {
        Some(Expression::List(v)) => v,
        Some(e) => vec![e],
        None => vec![],
    };
    list.push(tail);
    Expression::List(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::{Lexeme, Token};
    use ast::number_value::NumberValue;
    use context::Context;

    fn semicolon() -> Lexeme {
        Lexeme { token: Token::Semicolon, offset: None }
    }

    fn blue() -> Lexeme {
        Lexeme { token: Token::String("blue".into()), offset: None }
    }

    fn plus() -> Lexeme {
        Lexeme { token: Token::Plus, offset: None }
    }

    fn slash() -> Lexeme {
        Lexeme { token: Token::Slash, offset: None }
    }

    fn one_lexeme() -> Lexeme {
        Lexeme { token: Token::Number(1.0, None), offset: None }
    }

    fn one() -> NumberValue {
        NumberValue::from_scalar(one_lexeme())
    }

    fn two_lexeme() -> Lexeme {
        Lexeme { token: Token::Number(2.0, None), offset: None }
    }

    fn two() -> NumberValue {
        NumberValue::from_scalar(two_lexeme())
    }

    fn one_px_lexeme() -> Lexeme {
        Lexeme { token: Token::Number(1.0, Some("px".into())), offset: None }
    }

    fn one_px() -> NumberValue {
        NumberValue::from_scalar(one_px_lexeme())
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
            Ok(one_lexeme()),
            Ok(plus()),
            Ok(one_lexeme()),
            Ok(semicolon())
        ].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::List(vec![
                Expression::Number(one()),
                Expression::Operator(plus()),
                Expression::Number(one()),
            ]))
        );
    }

    #[test]
    fn it_parses_a_number_without_units() {
        let mut fake_tokenizer = vec![
            Ok(one_lexeme()),
            Ok(semicolon())
        ].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Number(one()))
        );
    }

    #[test]
    fn it_parses_a_number_with_units() {
        let mut fake_tokenizer = vec![
            Ok(one_px_lexeme()),
            Ok(semicolon())
        ].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Number(one_px()))
        );
    }

    #[test]
    fn it_evaluates_a_list() {
        let ex = Expression::List(vec![
            Expression::Number(one()),
            Expression::Operator(slash()),
            Expression::Number(two()),
            Expression::Operator(plus()),
            Expression::Number(one()),
            Expression::Operator(slash()),
            Expression::Number(two()),
        ]);
        let fake_context = Context::new();
        assert_eq!(
            ex.evaluate(&fake_context),
            Expression::Number(NumberValue {
                scalar: 1.0,
                computed: true,
                units: None,
                offset: None,
            })
        );
    }
}
