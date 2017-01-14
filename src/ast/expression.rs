use token::Token;
use token_offset::TokenOffset;
use operator::Operator;
use operator_offset::OperatorOffset;
use operator_or_token::OperatorOrToken;
use context::Context;
use error::{Result, SassError, ErrorKind};
use expression_evaluator::ExpressionEvaluator;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    List(Vec<Expression>),
    Value(OperatorOrToken),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::List(ref elements) => {
                elements.iter().map(|e|
                    e.to_string()
                ).collect::<Vec<_>>()
                 .join(" ")
                 .replace(" / ", "/")
                 .fmt(f)
            },
            Expression::Value(ref v) => v.fmt(f),
        }
    }
}

impl Expression {
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

    fn apply_slash(first: OperatorOrToken, second: OperatorOrToken, paren_level: i32, offset: Option<usize>) -> Expression {
        if paren_level == 0 {
            debug!("Paren level 0. First computed: {}, second computed: {}", first.computed_number(), second.computed_number());
            if first.computed_number() || second.computed_number() {
                Expression::Value(first / second)
            } else {
                Expression::List(vec![
                    Expression::Value(first),
                    Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                        operator: Operator::Slash,
                        offset: offset,
                    })),
                    Expression::Value(second),
                ])
            }
        } else {
            debug!("Paren level {}", paren_level);
            Expression::Value(first / second)
        }
    }

    fn force_list_collapse(list: Vec<Expression>, context: &Context) -> Expression {
        if list.iter().any(|item| {
            match *item {
                Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Slash, ..
                })) => true,
                _ => false,
            }
        }) {
            let mut evaluator = ExpressionEvaluator::new(context);
            evaluator.paren_level = 1;
            evaluator.evaluate_list(list)
        } else {
            Expression::List(list)
        }
    }

    pub fn apply_math(operator: OperatorOffset, first: Expression, second: Expression, context: &Context, paren_level: i32) -> Expression {
        debug!("Applying math to:\nfirst: {:#?}\nop: {:#?}\nsecond: {:#?}", first, operator, second);

        match (first, second) {
            (Expression::Value(f), Expression::Value(s)) => {
                let result = match operator.operator {
                    Operator::Plus => f + s,
                    Operator::Minus => f - s,
                    Operator::Star => f * s,
                    Operator::Percent => f % s,
                    Operator::Slash => return Expression::apply_slash(
                        f, s, paren_level, operator.offset
                    ),
                    _ => unimplemented!(),
                };
                Expression::Value(result)
            },
            (Expression::List(f), Expression::List(s)) => {
                let eval_first = Expression::force_list_collapse(f, context);
                let eval_second = Expression::force_list_collapse(s, context);

                match (eval_first, eval_second) {
                    (Expression::List(mut fi), Expression::List(se)) => {
                        match operator.operator {
                            Operator::Plus | Operator::Comma => {
                                fi.extend(se);
                                Expression::List(fi)
                            },
                            _ => panic!("Can't use an operator other than \
                                         plus or comma on two lists"),
                        }
                    },
                    (eval_first, eval_second) => {
                        Expression::apply_math(
                            operator, eval_first, eval_second,
                            context, paren_level
                        )
                    }
                }
            },
            (Expression::List(f), Expression::Value(s)) => {
                let mut first_evaluator = ExpressionEvaluator::new(context);
                first_evaluator.paren_level = paren_level;
                let eval_first = first_evaluator.evaluate_list(f);

                match eval_first {
                    Expression::List(mut fi) => {
                        match operator.operator {
                            Operator::Plus => {
                                fi.push(Expression::Value(s));
                                Expression::List(fi)
                            },
                            Operator::Slash => {
                                if s.computed_number() {
                                    let forced = Expression::force_list_collapse(
                                        fi,
                                        context
                                    );

                                    match forced {
                                        Expression::List(mut fi) => {
                                            fi.push(
                                                Expression::Value(
                                                    OperatorOrToken::Operator(
                                                        operator
                                                    )
                                                )
                                            );
                                            fi.push(Expression::Value(s));
                                            Expression::List(fi)
                                        },
                                        Expression::Value(fo) => {
                                            Expression::Value(fo / s)
                                        }
                                    }
                                } else {
                                    fi.push(
                                        Expression::Value(
                                            OperatorOrToken::Operator(operator)
                                        )
                                    );
                                    fi.push(Expression::Value(s));
                                    Expression::List(fi)
                                }
                            },
                            _ => panic!("Can't use an operator other than \
                                         plus on a list and a value"),
                        }
                    },
                    _ => Expression::apply_math(
                        operator, eval_first, Expression::Value(s),
                        context, paren_level
                    ),
                }
            },
            (Expression::Value(f), Expression::List(s)) => {
                debug!("Value Op List: {:#?}\n{:#?}\n{:#?}\n", f, operator, s);
                let eval_second = Expression::force_list_collapse(s, context);
                match eval_second {
                    Expression::List(se) => {
                        match operator.operator {
                            Operator::Plus => {
                                let (first_in_list, rest) = se.split_first()
                                    .expect("Trying to get the first and rest \
                                          of a list that isn't a value failed");
                                let new_first = format!("{}{}", f, first_in_list);
                                let mut new_list = vec![
                                    Expression::Value(OperatorOrToken::Token(
                                        TokenOffset {
                                            offset: f.offset(),
                                            token: Token::String(new_first),
                                        }
                                    ))
                                ];
                                new_list.extend_from_slice(rest);
                                Expression::List(new_list)
                            },
                            _ => panic!("Can't use an operator other than \
                                         plus on a value and a list"),
                        }
                    },
                    _ => Expression::apply_math(
                        operator, Expression::Value(f), eval_second,
                        context, paren_level
                    ),
                }
            },
        }
    }

    pub fn create_list(head: Option<Expression>, tail: Expression) -> Expression {
        let mut list = match head {
            Some(Expression::List(v)) => v,
            Some(e) => vec![e],
            None => vec![],
        };
        list.push(tail);
        Expression::List(list)
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Expression::Value(OperatorOrToken::Token(TokenOffset {
                token: Token::Number { .. }, ..
            })) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match *self {
            Expression::Value(OperatorOrToken::Token(TokenOffset {
                token: Token::String(_), ..
            })) => true,
            _ => false,
        }
    }

    pub fn is_right_paren(&self) -> bool {
        match *self {
            Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                operator: Operator::RightParen, ..
            })) => true,
            _ => false,
        }
    }

    pub fn is_left_paren(&self) -> bool {
        match *self {
            Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                operator: Operator::LeftParen, ..
            })) => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        match *self {
            Expression::Value(OperatorOrToken::Operator(_)) => true,
            _ => false,
        }
    }

    pub fn extract_operator_offset(self) -> OperatorOffset {
        match self {
            Expression::Value(OperatorOrToken::Operator(operator_offset)) => {
                operator_offset
            },
            _ => panic!("Can't extract operator offset from {:?}", self),
        }
    }

    pub fn extract_token_offset(self) -> TokenOffset {
        match self {
            Expression::Value(OperatorOrToken::Token(token_offset)) => {
                token_offset
            },
            _ => panic!("Can't extract token offset from {:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::Token;
    use token_offset::TokenOffset;
    use operator_or_token::OperatorOrToken;
    use operator::Operator;
    use operator_offset::OperatorOffset;

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
}
