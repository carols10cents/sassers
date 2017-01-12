use token::Token;
use context::Context;
use operator::Operator;
use operator_offset::OperatorOffset;
use operator_or_token::OperatorOrToken;
use token_offset::TokenOffset;
use ast::expression::Expression;

pub struct ExpressionEvaluator<'a> {
    context: &'a Context,
    pub paren_level: i32,
    last_was_an_operator: bool,
    value_stack: Vec<Expression>,
    op_stack: Vec<OperatorOffset>,
}

impl<'a> ExpressionEvaluator<'a> {
    pub fn evaluate(expr: Expression, context: &Context) -> Expression {
        match expr {
            Expression::Value(OperatorOrToken::Token(t @ TokenOffset {
                token: Token::String(_), ..
            })) => {
                context.get_variable(&t)
                       .unwrap_or(Expression::Value(
                            OperatorOrToken::Token(t)
                       ))
            },
            Expression::List(exprs) => {
                let evaluator = ExpressionEvaluator::new(context);
                evaluator.evaluate_list(exprs)
            },
            other => other,
        }
    }

    pub fn new(context: &Context) -> ExpressionEvaluator {
        ExpressionEvaluator {
            context: context,
            paren_level: 0,
            last_was_an_operator: true,
            value_stack: Vec::new(),
            op_stack: Vec::new(),
        }
    }

    pub fn evaluate_list(mut self, exprs: Vec<Expression>) -> Expression {

        // Split into value stacks and operator stacks
        let mut exprs = exprs.into_iter();

        while let Some(part) = exprs.next() {

            debug!("Processing list item {:#?}", part);

            match part {
                v @ Expression::Value(OperatorOrToken::Token(TokenOffset {
                    token: Token::Number { .. }, ..
                })) => {
                    if self.last_was_an_operator {
                        debug!("Push on value stack {:#?}", v);
                        self.value_stack.push(v);
                    } else {
                        debug!("Number, last_was_an_operator=false, paren_level={}", self.paren_level);

                        if self.paren_level > 0 {
                            while !self.op_stack.is_empty() &&
                                  self.op_stack.last().unwrap().operator !=
                                      Operator::LeftParen {

                                debug!("op stack last {:#?}", self.op_stack.last());
                                self.math_machine();
                            }
                        }

                        let list = Expression::create_list(
                            self.value_stack.pop(),
                            v,
                        );

                        debug!("Push on list on value stack {:#?}", list);

                        self.value_stack.push(list);
                    }

                    self.last_was_an_operator = false;
                },
                Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::RightParen, ..
                })) => {
                    debug!("RIGHT PAREN");
                    debug!("op stack = {:#?}", self.op_stack);

                    let mut last_operator = self.op_stack.pop();
                    while last_operator.is_some() &&
                          last_operator.unwrap().operator !=
                              Operator::LeftParen {

                        debug!("last operator before right paren = {:#?}", last_operator);
                        self.op_stack.push(last_operator.unwrap());
                        self.math_machine();
                        last_operator = self.op_stack.pop();
                    }
                    self.last_was_an_operator = false;
                    self.paren_level -= 1;
                },
                Expression::Value(OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::LeftParen, offset
                })) => {
                    debug!("Push on op stack Leftparen");
                    self.op_stack.push(OperatorOffset {
                        operator: Operator::LeftParen, offset: offset
                    });
                    self.last_was_an_operator = true;
                    self.paren_level += 1;
                },
                Expression::Value(OperatorOrToken::Operator(
                    oo @ OperatorOffset { .. }
                )) => {
                    if let Some(&last_operator) = self.op_stack.last() {
                        if last_operator.operator
                                .same_or_greater_precedence(oo.operator) {
                            self.math_machine();
                        }
                    }
                    debug!("Push on op stack {:#?}", oo);
                    self.op_stack.push(oo);
                    self.last_was_an_operator = true;
                },
                Expression::Value(OperatorOrToken::Token(t @ TokenOffset {
                    token: Token::String(_), ..
                })) => {
                    let var_eval = self.context.get_variable(&t)
                                    .unwrap_or(Expression::Value(
                                        OperatorOrToken::Token(t)
                                    ));

                    if self.last_was_an_operator {
                        self.value_stack.push(var_eval);
                    } else {
                        let list = Expression::create_list(
                            self.value_stack.pop(),
                            var_eval,
                        );
                        self.value_stack.push(list);
                    }
                    self.last_was_an_operator = false;
                },
                Expression::List(list) => {
                    debug!("Push list on value stack {:#?}", list);
                    self.value_stack.push(Expression::List(list));
                    self.last_was_an_operator = false;
                },
                other => {
                    if self.last_was_an_operator {
                        self.value_stack.push(other);
                    } else {
                        let list = Expression::create_list(
                            self.value_stack.pop(),
                            other,
                        );
                        self.value_stack.push(list);
                    }
                    self.last_was_an_operator = false;
                },
            }
        }

        debug!("PROCESS THE STACKS!");
        debug!("Op stack = {:#?}", self.op_stack);
        debug!("Value stack = {:#?}", self.value_stack);

        // Process the stacks
        while !self.op_stack.is_empty() {
            self.math_machine()
        }

        self.value_stack.pop().unwrap()
    }

    fn math_machine(&mut self) {
        debug!("Math machine:");

        let op = self.op_stack.pop().unwrap();
        debug!("op = {:#?}", op);

        let second = self.value_stack.pop()
                       .expect("Expected a second argument on the value stack");
        debug!("second = {:#?}", second);

        let first = self.value_stack.pop()
                       .expect("Expected a first argument on the value stack");
        debug!("first: {:#?}", first);

        let math_result = Expression::apply_math(
            op, first, second, self.context, self.paren_level,
        );
        debug!("Math result: {:#?}", math_result);

        self.value_stack.push(math_result);
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
    use ast::expression::Expression;

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
            ExpressionEvaluator::evaluate(ex, &fake_context),
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
            ExpressionEvaluator::evaluate(ex, &fake_context),
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
            ExpressionEvaluator::evaluate(ex, &fake_context),
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
