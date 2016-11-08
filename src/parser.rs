use token::Token;
use token_offset::TokenOffset;
use operator_or_token::OperatorOrToken;
use operator::Operator;
use operator_offset::OperatorOffset;
use tokenizer::Tokenizer;
use ast::expression::Expression;
use ast::root::Root;
use ast::node::Node;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::comment::SassComment;
use error::{Result, SassError, ErrorKind};

pub struct Parser<'a> {
    pub tokenizer: Tokenizer<'a>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Root>;

    fn next(&mut self) -> Option<Result<Root>> {
        let mut current_sass_rule = SassRule::new();
        let mut ambiguous_holding_pen: Vec<TokenOffset> = vec![];

        while let Some(Ok(op_or_token)) = self.tokenizer.next() {
            match op_or_token {
                OperatorOrToken::Token(TokenOffset {
                    token: Token::String(ref string_val), offset: off,
                }) if string_val.starts_with("$") => {
                    let variable_name = TokenOffset {
                        token: Token::String((*string_val).clone()),
                        offset: off,
                    };

                    if let Err(e) = Parser::expect_operator(
                        Operator::Colon,
                        self.tokenizer.next()
                    ) { return Some(Err(e)) };

                    let variable_value = match Expression::parse(&mut self.tokenizer) {
                        Ok(e) => e,
                        Err(e) => return Some(Err(e)),
                    };
                    return Some(Ok(Root::Variable(
                        SassVariable {
                            name: variable_name,
                            value: variable_value
                        }
                    )))
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::LeftCurlyBrace, ..
                }) => {
                    current_sass_rule.selectors.extend_from_slice(
                        &ambiguous_holding_pen[..]
                    );
                    current_sass_rule.children = match self.parse_body() {
                        Ok(body) => body,
                        Err(e) => return Some(Err(e)),
                    };
                    return Some(Ok(Root::Rule(current_sass_rule)))
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Comma, ..
                }) => {
                    current_sass_rule.selectors.extend_from_slice(
                        &ambiguous_holding_pen
                    );
                    ambiguous_holding_pen = vec![];
                },
                OperatorOrToken::Token(t @ TokenOffset {
                    token: Token::Comment(_), ..
                }) => {
                    if ambiguous_holding_pen.is_empty() {
                        return Some(Ok(Root::Comment(
                            SassComment { content: t }
                        )))
                    }
                },
                _ => {
                    match ambiguous_holding_pen.pop() {
                        Some(held) => {
                            ambiguous_holding_pen.push(
                                held.combine(&op_or_token)
                            );
                        },
                        None => {
                            ambiguous_holding_pen = vec![op_or_token.into()];
                        },
                    }
                }
            }
        }
        None
    }
}

impl<'a> Parser<'a> {
    pub fn new(text: &str) -> Parser {
        Parser {
            tokenizer: Tokenizer::new(&text),
        }
    }

    pub fn expect_operator(expected: Operator, actual: Option<Result<OperatorOrToken>>) -> Result<OperatorOrToken> {
        match actual {
            None => Err(SassError {
                offset: 0,
                kind: ErrorKind::UnexpectedEof,
                message: format!(
                    "Expected to see `{}`, instead reached EOF.",
                    expected,
                ),
            }),
            Some(res) => {
                match res {
                    Err(e) => Err(e),
                    Ok(OperatorOrToken::Operator(OperatorOffset {
                        operator: ref actual_operator, offset: off
                    })) if *actual_operator == expected => {
                        Ok(OperatorOrToken::Operator(OperatorOffset {
                            operator: *actual_operator, offset: off
                        }))
                    },
                    Ok(OperatorOrToken::Token(TokenOffset {
                        token: actual_token, offset
                    })) => {
                        Err(SassError {
                            offset: offset.unwrap_or(0),
                            kind: ErrorKind::ParserError,
                            message: format!(
                                "Expected to see `{}`, instead saw `{}`.",
                                expected,
                                actual_token,
                            ),
                        })
                    },
                    Ok(OperatorOrToken::Operator(OperatorOffset {
                        operator: actual_operator, offset
                    })) => {
                        Err(SassError {
                            offset: offset.unwrap_or(0),
                            kind: ErrorKind::ParserError,
                            message: format!(
                                "Expected to see `{}`, instead saw `{}`.",
                                expected,
                                actual_operator,
                            ),
                        })
                    }
                }
            }
        }
    }

    pub fn parse_body(&mut self) -> Result<Vec<Node>> {
        let mut rule_stack = vec![];
        let mut body = vec![];
        let mut ambiguous_holding_pen = vec![];

        while let Some(Ok(op_or_token)) = self.tokenizer.next() {
            match op_or_token {
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::RightCurlyBrace, ..
                }) => {
                    if rule_stack.is_empty() {
                        return Ok(body);
                    } else {
                        let tmp_rule = rule_stack.pop().unwrap();

                        if rule_stack.is_empty() {
                            body.push(Node::Rule(tmp_rule));
                        } else {
                            // TODO: mut ref to last?
                            let mut rule = rule_stack.pop().unwrap();
                            rule.children.push(Node::Rule(tmp_rule));
                            rule_stack.push(rule);
                        }
                    }
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::LeftCurlyBrace, ..
                }) => {
                    let mut rule = SassRule::new();
                    rule.selectors = ambiguous_holding_pen;
                    ambiguous_holding_pen = vec![];
                    rule_stack.push(rule);
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Colon, ..
                }) => {
                    let value = try!(Expression::parse(&mut self.tokenizer));

                    let child = match ambiguous_holding_pen.pop() {
                        Some(name_lexeme) => {
                            match name_lexeme.token {
                                Token::String(ref s) if s.starts_with("$") => {
                                    Node::Variable(
                                        SassVariable {
                                            name: name_lexeme.clone(),
                                            value: value,
                                        }
                                    )
                                },
                                Token::String(_) => {
                                    Node::Property(name_lexeme, value)
                                },
                                other => {
                                    return Err(SassError {
                                        offset: name_lexeme.offset.unwrap_or(0),
                                        kind: ErrorKind::ParserError,
                                        message: format!(
                                            "Expected to have seen a property or variable name, instead saw {:?}", other
                                        ),
                                    })
                                },
                            }
                        },
                        None => {
                            return Err(SassError {
                                offset: op_or_token.offset().unwrap_or(0),
                                kind: ErrorKind::ParserError,
                                message: String::from(
                                    "Expected to have seen a property or \
                                     variable name, did not see any"
                                ),
                            })
                        }
                    };
                    if rule_stack.is_empty() {
                        body.push(child);
                    } else {
                        // TODO: mut ref to last?
                        let mut rule = rule_stack.pop().unwrap();
                        rule.children.push(child);
                        rule_stack.push(rule);
                    }
                },
                OperatorOrToken::Operator(OperatorOffset {
                    operator: Operator::Comma, ..
                }) => {
                    // TODO: else return error
                    if let Some(Ok(after_comma)) = self.tokenizer.next() {
                        ambiguous_holding_pen.push(after_comma.into());
                    }
                },
                OperatorOrToken::Token(content @ TokenOffset {
                    token: Token::Comment(_), ..
                }) => {
                    if rule_stack.is_empty() &&
                       ambiguous_holding_pen.is_empty() {
                        body.push(
                            Node::Comment(
                                SassComment { content: content }
                            )
                        );
                    } else if !rule_stack.is_empty() {
                        // TODO: mut ref to last?
                        let mut rule = rule_stack.pop().unwrap();
                        rule.children.push(Node::Comment(
                            SassComment { content: content }
                        ));
                        rule_stack.push(rule);
                    }
                },
                _ => {
                    match ambiguous_holding_pen.pop() {
                        Some(held) => {
                            ambiguous_holding_pen.push(
                                held.combine(&op_or_token)
                            );
                        },
                        None => {
                            ambiguous_holding_pen = vec![op_or_token.into()];
                        },
                    }
                },
            }
        }

        Err(SassError {
            offset: 0,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected to see rule body ending in `}`, instead reached EOF."
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::rule::SassRule;
    use sass::comment::SassComment;
    use sass::variable::SassVariable;
    use ast::expression::Expression;
    use ast::root::Root;
    use ast::node::Node;
    use token::Token;
    use token_offset::TokenOffset;
    use operator_or_token::OperatorOrToken;
    use error::{SassError, ErrorKind};

    #[test]
    fn it_returns_none_for_empty_string() {
        let mut parser = Parser::new("");
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_a_rule() {
        let mut parser = Parser::new("a { color: blue; }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![TokenOffset { token: Token::String("a".into()), offset: Some(0) }],
                children: vec![Node::Property(
                    TokenOffset { token: Token::String("color".into()), offset: Some(4) },
                    Expression::Value(OperatorOrToken::Token(
                        TokenOffset {
                            token: Token::String("blue".into()),
                            offset: Some(11),
                        }
                    )),
                )],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_nested_rules() {
        let mut parser = Parser::new("div { span img, span a { color: blue; } }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![TokenOffset { token: Token::String("div".into()), offset: Some(0) }],
                children: vec![Node::Rule(
                    SassRule {
                        selectors: vec![
                            TokenOffset {
                                token: Token::String("span img".into()),
                                offset: Some(6)
                            },
                            TokenOffset {
                                token: Token::String("span a".into()),
                                offset: Some(16)
                            },
                        ],
                        children: vec![Node::Property(
                            TokenOffset { token: Token::String("color".into()), offset: Some(25) },
                            Expression::Value(OperatorOrToken::Token(
                                TokenOffset {
                                    token: Token::String("blue".into()),
                                    offset: Some(32),
                                }
                            )),
                        )],
                    }
                )],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_rules_without_properties() {
        let mut parser = Parser::new("div { empty { span { color: red; } } }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![
                    TokenOffset {
                        token: Token::String("div".into()),
                        offset: Some(0)
                    },
                ],
                children: vec![
                    Node::Rule(
                        SassRule {
                            selectors: vec![
                                TokenOffset {
                                    token: Token::String("empty".into()),
                                    offset: Some(6)
                                },
                            ],
                            children: vec![
                                Node::Rule(
                                    SassRule {
                                        selectors: vec![
                                            TokenOffset {
                                                token: Token::String("span".into()),
                                                offset: Some(14)
                                            }
                                        ],
                                        children: vec![
                                            Node::Property(
                                                TokenOffset {
                                                    token: Token::String("color".into()),
                                                    offset: Some(21)
                                                },
                                                Expression::Value(OperatorOrToken::Token(
                                                    TokenOffset {
                                                        token: Token::String("red".into()),
                                                        offset: Some(28)
                                                    }
                                                ))
                                            )
                                        ],
                                    }
                                ),
                            ],
                        }
                    ),
                ]
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_rules_with_multiple_selectors() {
        let mut parser = Parser::new("a, b c { color: red; }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![
                    TokenOffset { token: Token::String("a".into()), offset: Some(0) },
                    TokenOffset { token: Token::String("b c".into()), offset: Some(3) },
                ],
                children: vec![Node::Property(
                    TokenOffset { token: Token::String("color".into()), offset: Some(9) },
                    Expression::Value(OperatorOrToken::Token(
                        TokenOffset {
                            token: Token::String("red".into()),
                            offset: Some(16),
                        }
                    )),
                )],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_variable_setting_statements() {
        let mut parser = Parser::new("$color: red;");
        assert_eq!(parser.next(), Some(Ok(Root::Variable(SassVariable {
            name: TokenOffset { token: Token::String("$color".into()), offset: Some(0) },
            value: Expression::Value(OperatorOrToken::Token(
                TokenOffset {
                    token: Token::String("red".into()),
                    offset: Some(8),
                }
            )),
        }))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_variables_set_within_rules() {
        let mut parser = Parser::new("a { $foo: red; }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![
                    TokenOffset { token: Token::String("a".into()), offset: Some(0) },
                ],
                children: vec![Node::Variable(SassVariable {
                    name: TokenOffset { token: Token::String("$foo".into()), offset: Some(4) },
                    value: Expression::Value(OperatorOrToken::Token(
                        TokenOffset {
                            token: Token::String("red".into()),
                            offset: Some(10),
                        }
                    )),
                })],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_a_comment() {
        let mut parser = Parser::new("/* hi */");
        assert_eq!(parser.next(), Some(Ok(Root::Comment(SassComment {
            content: TokenOffset { token: Token::Comment("/* hi */".into()), offset: Some(0) },
        }))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_errors_with_malformed_variable_declaration() {
        let mut parser = Parser::new("$var no-colon;");
        assert_eq!(parser.next(), Some(Err(SassError {
            offset: 5,
            kind: ErrorKind::ParserError,
            message: String::from("Expected to see `:`, instead saw `no-colon`."),
        })));
    }
}
