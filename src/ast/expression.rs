use sass::output_style::SassOutputStyle;
use token::{Token, Operator, OperatorOffset, TokenOffset, OperatorOrToken};
use context::Context;
use error::{Result, SassError, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    List(Vec<Expression>),
    Value(OperatorOrToken),
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
            Expression::Value(ref v) => v.to_string(),
        }
    }

    fn parse_parenthetical<T>(tokenizer: &mut T) -> Result<Expression>
        where T: Iterator<Item = Result<OperatorOrToken>> {

        let mut list = vec![];
        while let Some(Ok(t)) = tokenizer.next() {
            match t {
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::RightParen, ..
                }) => {
                    return Ok(Expression::List(list))
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::LeftParen, ..
                }) => {
                    list.push(
                        try!(Expression::parse_parenthetical(tokenizer))
                    );
                },
                OperatorOrToken::Token(TokenOffset {
                    token: Token::Comment(_), ..
                }) => {},
                _ => list.push(Expression::Value(t)),
            }
        }

        let error_offset = match list.pop() {
            Some(Expression::Value(v)) => v.offset().unwrap_or(0),
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

    pub fn parse<T>(tokenizer: &mut T) -> Result<Expression>
        where T: Iterator<Item = Result<OperatorOrToken>> {
        let mut list = vec![];
        while let Some(Ok(t)) = tokenizer.next() {
            match t {
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Semicolon, ..
                }) => {
                    if list.len() == 1 {
                        return Ok(list.pop().unwrap())
                    } else {
                        return Ok(Expression::List(list))
                    }
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::LeftParen, ..
                }) => {
                    list.push(
                        try!(Expression::parse_parenthetical(tokenizer))
                    );
                },
                OperatorOrToken::Token(TokenOffset {
                    token: Token::Comment(_), ..
                }) => {},
                _ => list.push(Expression::Value(t)),
            }
        }

        let error_offset = match list.pop() {
            Some(Expression::Value(v)) => v.offset().unwrap_or(0),
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
        let mut op_stack: Vec<OperatorOffset> = Vec::new();

        // Split into value stacks and operator stacks

        let mut exprs = exprs.into_iter();
        while let Some(part) = exprs.next() {
            debug!("Processing list item {:#?}", part);
            match part {
                v @ Expression::Value(OperatorOrToken::Token(TokenOffset {
                    token: Token::Number { .. }, ..
                })) => {
                    if last_was_an_operator {
                        debug!("Push on value stack {:#?}", v);
                        value_stack.push(v);
                    } else {
                        let list = Expression::create_list(
                            value_stack.pop(),
                            v,
                        );
                        debug!("Push on list on value stack {:#?}", list);
                        value_stack.push(list);
                    }
                    last_was_an_operator = false;
                },
                Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Slash, offset: off,
                })) if !force_slash => {
                    let list = Expression::create_list(
                        value_stack.pop(),
                        Expression::Value(OperatorOrToken::Operator(
                            OperatorOffset {
                                operator: Operator::Slash, offset: off,
                            }
                        )),
                    );
                    debug!("Push on list on value stack {:#?}", list);
                    value_stack.push(list);
                },
                Expression::Value(OperatorOrToken::Operator(
                    oo @ OperatorOffset { .. }
                )) => {
                    debug!("Push on op stack {:#?}", oo);
                    op_stack.push(oo);
                    last_was_an_operator = true;
                },
                Expression::Value(OperatorOrToken::Token(t @ TokenOffset {
                    token: Token::String(_), ..
                })) => {
                    let var_eval = context.get_variable(&t)
                                    .unwrap_or(Expression::Value(
                                        OperatorOrToken::Token(t)
                                    ));

                    if last_was_an_operator {
                        value_stack.push(var_eval);
                    } else {
                        let list = Expression::create_list(
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
                other => {
                    if last_was_an_operator {
                        value_stack.push(other);
                    } else {
                        let list = Expression::create_list(
                            value_stack.pop(),
                            other,
                        );
                        value_stack.push(list);
                    }
                    last_was_an_operator = false;
                },
            }
        }

        // Process the stacks
        while !op_stack.is_empty() {
            let op = op_stack.pop().unwrap();
            let second = value_stack.pop().expect("Expected a second argument on the value stack");
            let first = value_stack.pop().expect("Expected a first argument on the value stack");

            let math_result = Expression::apply_math(
                op, first, second, context
            );
            debug!("Math result: {:#?}", math_result);

            value_stack.push(math_result);
        }

        value_stack.pop().unwrap()
    }

    pub fn evaluate(self, context: &Context) -> Expression {
        match self {
            Expression::Value(OperatorOrToken::Token(t @ TokenOffset {
                token: Token::String(_), ..
            })) => {
                context.get_variable(&t)
                       .unwrap_or(Expression::Value(
                            OperatorOrToken::Token(t)
                       ))
            },
            Expression::List(exprs) => {
                Expression::evaluate_list(exprs, context, false)
            },
            other => other,
        }
    }

    fn apply_math(operator: OperatorOffset, first: Expression, second: Expression, context: &Context) -> Expression {
        debug!("Applying math to:\nfirst: {:#?}\nop: {:#?}\nsecond: {:#?}", first, operator, second);

        match (first, second) {
            (Expression::Value(f), Expression::Value(s)) => {
                let result = match operator.operator {
                    Operator::Plus => f + s,
                    Operator::Minus => f - s,
                    Operator::Star => f * s,
                    Operator::Slash => f / s,
                    Operator::Percent => f % s,
                    _ => unimplemented!(),
                };
                Expression::Value(result)
            },
            (Expression::List(f), Expression::List(s)) => {
                let eval_first = Expression::evaluate_list(f, context, true);
                let eval_second = Expression::evaluate_list(s, context, true);
                Expression::apply_math(
                    operator, eval_first, eval_second, context
                )
            },
            (Expression::List(f), Expression::Value(s)) => {
                let eval_first = Expression::evaluate_list(f, context, true);
                Expression::apply_math(
                    operator, eval_first, Expression::Value(s), context
                )
            },
            (Expression::Value(f), Expression::List(s)) => {
                if s.iter().any(|e|
                    match *e {
                        Expression::Value(
                            OperatorOrToken::Operator(OperatorOffset {
                                operator: Operator::Slash, ..
                            })
                        ) => true,
                        _ => false,
                    }) {
                    let evaled_list = Expression::evaluate_list(
                        s, context, true
                    );
                    Expression::apply_math(
                        operator, Expression::Value(f), evaled_list, context
                    )
                } else {
                    if let Some((ref first_in_list, rest_of_list)) =
                            s.split_first() {

                        let new_first = Expression::apply_math(
                            operator,
                            Expression::Value(f),
                            (*first_in_list).clone(),
                            context
                        );

                        let mut new_list = vec![new_first];
                        new_list.extend_from_slice(rest_of_list);
                        Expression::List(new_list)
                    } else {
                        panic!("Trying to perform an operation on a number and a list; expected to get a list with something in it");
                    }
                }
            },
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::{OperatorOrToken, OperatorOffset, Operator, TokenOffset, Token};
    use context::Context;

    fn semicolon() -> OperatorOrToken {
        OperatorOrToken::Operator(
            OperatorOffset { operator: Operator::Semicolon, offset: None }
        )
    }

    fn blue() -> OperatorOrToken {
        OperatorOrToken::Token(
            TokenOffset { token: Token::String("blue".into()), offset: None }
        )
    }

    fn plus() -> OperatorOrToken {
        OperatorOrToken::Operator(
            OperatorOffset { operator: Operator::Plus, offset: None }
        )
    }

    fn slash() -> OperatorOrToken {
        OperatorOrToken::Operator(
            OperatorOffset { operator: Operator::Slash, offset: None }
        )
    }

    fn one() -> OperatorOrToken {
        OperatorOrToken::Token(
            TokenOffset {
                token: Token::Number {
                    value: 1.0, units: None, computed: false
                },
                offset: None
            }
        )
    }

    fn two() -> OperatorOrToken {
        OperatorOrToken::Token(
            TokenOffset {
                token: Token::Number {
                    value: 2.0, units: None, computed: false
                },
                offset: None
            }
        )
    }

    fn one_px() -> OperatorOrToken {
        OperatorOrToken::Token(
            TokenOffset {
                token: Token::Number {
                    value: 1.0, units: Some("px".into()), computed: false
                },
                offset: None
            }
        )
    }

    #[test]
    fn it_parses_a_single_string() {
        let mut fake_tokenizer = vec![Ok(blue()), Ok(semicolon())].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Value(blue()))
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
                Expression::Value(one()),
                Expression::Value(plus()),
                Expression::Value(one()),
            ]))
        );
    }

    #[test]
    fn it_parses_a_number_without_units() {
        let mut fake_tokenizer = vec![
            Ok(one()),
            Ok(semicolon())
        ].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Value(one()))
        );
    }

    #[test]
    fn it_parses_a_number_with_units() {
        let mut fake_tokenizer = vec![
            Ok(one_px()),
            Ok(semicolon())
        ].into_iter();
        assert_eq!(
            Expression::parse(&mut fake_tokenizer),
            Ok(Expression::Value(one_px()))
        );
    }

    #[test]
    fn it_evaluates_a_list() {
        let ex = Expression::List(vec![
            Expression::Value(one()),
            Expression::Value(slash()),
            Expression::Value(two()),
            Expression::Value(plus()),
            Expression::Value(one()),
            Expression::Value(slash()),
            Expression::Value(two()),
        ]);
        let fake_context = Context::new();
        assert_eq!(
            ex.evaluate(&fake_context),
            Expression::Value(OperatorOrToken::Token(
                TokenOffset {
                    token: Token::Number {
                        value: 1.0,
                        units: None,
                        computed: true,
                    },
                    offset: None,
                }
            ))
        );
    }
}
