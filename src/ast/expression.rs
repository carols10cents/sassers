use token::Token;
use token_offset::TokenOffset;
use operator::Operator;
use operator_offset::OperatorOffset;
use operator_or_token::OperatorOrToken;
use context::Context;
use error::{Result, SassError, ErrorKind};

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
                ).collect::<Vec<_>>().join(" ").fmt(f)
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

    fn evaluate_list(exprs: Vec<Expression>, context: &Context, mut paren_level: i32) -> Expression {

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
                        debug!("Number, last_was_an_operator=false, paren_level={}", paren_level);
                        if paren_level > 0 {
                            while !op_stack.is_empty() &&
                                  op_stack.last().unwrap().operator !=
                                      Operator::LeftParen {
                                debug!("op stack last {:#?}", op_stack.last());
                                Expression::math_machine(
                                    &mut op_stack, &mut value_stack,
                                    context, paren_level
                                );
                            }
                        }
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
                    operator: Operator::RightParen, ..
                })) => {
                    debug!("RIGHT PAREN");
                    debug!("op stack = {:#?}", op_stack);
                    let mut last_operator = op_stack.pop();
                    while last_operator.is_some() &&
                          last_operator.unwrap().operator !=
                              Operator::LeftParen {
                        debug!("last operator before right paren = {:#?}", last_operator);
                        op_stack.push(last_operator.unwrap());
                        Expression::math_machine(
                            &mut op_stack, &mut value_stack,
                            context, paren_level
                        );
                        last_operator = op_stack.pop();
                    }
                    last_was_an_operator = false;
                    paren_level -= 1;
                },
                Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::LeftParen, offset
                })) => {
                    debug!("Push on op stack Leftparen");
                    op_stack.push(OperatorOffset {
                        operator: Operator::LeftParen, offset: offset
                    });
                    last_was_an_operator = true;
                    paren_level += 1;
                },
                Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Slash, offset
                })) if paren_level == 0 => {
                    debug!("Push on list on value stack slash");
                    let list = Expression::create_list(
                        value_stack.pop(),
                        Expression::Value(
                            OperatorOrToken::Operator(
                                OperatorOffset {
                                    operator: Operator::Slash, offset: offset
                                }
                            )
                        ),
                    );
                    value_stack.push(list);
                },
                Expression::Value(OperatorOrToken::Operator(
                    oo @ OperatorOffset { .. }
                )) => {
                    if let Some(&last_operator) = op_stack.last() {
                        if last_operator.operator
                                .same_or_greater_precedence(oo.operator) {
                            Expression::math_machine(
                                &mut op_stack, &mut value_stack,
                                context, paren_level
                            );
                        }
                    }
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

        debug!("PROCESS THE STACKS!");
        debug!("Op stack = {:#?}", op_stack);
        debug!("Value stack = {:#?}", value_stack);

        // Process the stacks
        while !op_stack.is_empty() {
            Expression::math_machine(
                &mut op_stack, &mut value_stack, context, paren_level
            );
        }

        value_stack.pop().unwrap()
    }

    fn math_machine(op_stack: &mut Vec<OperatorOffset>, value_stack: &mut Vec<Expression>, context: &Context, paren_level: i32) {
        let op = op_stack.pop().unwrap();
        let second = value_stack.pop()
                       .expect("Expected a second argument on the value stack");
        let first = value_stack.pop()
                       .expect("Expected a first argument on the value stack");

        debug!("Math machine: first: {:#?}\nop: {:#?}\nsecond: {:#?}",
            first, op, second
        );
        let math_result = Expression::apply_math(
            op, first, second, context, paren_level,
        );
        debug!("Math result: {:#?}", math_result);

        value_stack.push(math_result);
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
                Expression::evaluate_list(exprs, context, 0)
            },
            other => other,
        }
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
            Expression::evaluate_list(list, context, 1)
        } else {
            Expression::List(list)
        }
    }

    fn apply_math(operator: OperatorOffset, first: Expression, second: Expression, context: &Context, paren_level: i32) -> Expression {
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
                            Operator::Plus => {
                                fi.extend(se);
                                Expression::List(fi)
                            },
                            _ => panic!("Can't use an operator other than \
                                         plus on two lists"),
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
                let eval_first = Expression::evaluate_list(
                    f, context, paren_level
                );
                match eval_first {
                    Expression::List(mut fi) => {
                        match operator.operator {
                            Operator::Plus => {
                                fi.push(Expression::Value(s));
                                Expression::List(fi)
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
    use token::Token;
    use token_offset::TokenOffset;
    use operator_or_token::OperatorOrToken;
    use operator::Operator;
    use operator_offset::OperatorOffset;
    use context::Context;

    extern crate env_logger;

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

    fn right_paren() -> OperatorOrToken {
        OperatorOrToken::Operator(
            OperatorOffset { operator: Operator::RightParen, offset: None }
        )
    }

    fn left_paren() -> OperatorOrToken {
        OperatorOrToken::Operator(
            OperatorOffset { operator: Operator::LeftParen, offset: None }
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
    fn it_evaluates_a_list_adding_fractions() {
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

    #[test]
    fn it_evaluates_a_list_with_division_and_string_concat() {
        let _ = env_logger::init();

        let ex = Expression::List(vec![
            Expression::Value(one()),
            Expression::Value(plus()),
            Expression::Value(left_paren()),
            Expression::Value(one()),
            Expression::Value(slash()),
            Expression::Value(two()),
            Expression::Value(two()),
            Expression::Value(two()),
            Expression::Value(right_paren()),
        ]);
        let fake_context = Context::new();
        assert_eq!(
            ex.evaluate(&fake_context),
            Expression::List(vec![
                Expression::Value(OperatorOrToken::Token(
                    TokenOffset {
                        token: Token::String(String::from("10.5")),
                        offset: None,
                    }
                )),
                Expression::Value(OperatorOrToken::Token(
                    TokenOffset {
                        token: Token::Number {
                            value: 2.0,
                            units: None,
                            computed: false,
                        },
                        offset: None,
                    }
                )),
                Expression::Value(OperatorOrToken::Token(
                    TokenOffset {
                        token: Token::Number {
                            value: 2.0,
                            units: None,
                            computed: false,
                        },
                        offset: None,
                    }
                ))
            ])
        );
    }

    #[test]
    fn it_does_not_divide_list_with_only_slashes() {
        let ex = Expression::List(vec![
            Expression::Value(one()),
            Expression::Value(slash()),
            Expression::Value(one()),
            Expression::Value(slash()),
            Expression::Value(two()),
        ]);
        let fake_context = Context::new();
        assert_eq!(
            ex.evaluate(&fake_context),
            Expression::List(vec![
                Expression::Value(one()),
                Expression::Value(slash()),
                Expression::Value(one()),
                Expression::Value(slash()),
                Expression::Value(two()),
            ])
        );
    }
}
