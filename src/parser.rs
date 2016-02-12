use token::{Token, Lexeme};
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
        let mut rule_stack = vec![];
        let mut selector_holding_pen = Lexeme::new();

        while let Some(Ok(lexeme)) = self.tokenizer.next() {
            match lexeme.token {
                Token::String(ref s) if s.starts_with("$") => {
                    let variable_name = lexeme.clone();
                    // TODO: this better be a colon
                    self.tokenizer.next();
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
                Token::LeftCurlyBrace => {
                    current_sass_rule.selectors.push(selector_holding_pen);
                    let mut holding_pen = vec![];
                    while let Some(Ok(lexeme)) = self.tokenizer.next() {
                        match lexeme.token {
                            Token::RightCurlyBrace => {
                                if rule_stack.is_empty() {
                                    return Some(Ok(Root::Rule(current_sass_rule)))
                                } else {
                                    let tmp_rule = current_sass_rule;
                                    current_sass_rule = rule_stack.pop().unwrap();
                                    current_sass_rule.children.push(Node::Rule(tmp_rule));
                                }
                            },
                            Token::LeftCurlyBrace => {
                                rule_stack.push(current_sass_rule);
                                current_sass_rule = SassRule::new();
                                current_sass_rule.selectors = holding_pen;
                                holding_pen = vec![];
                            },
                            Token::Colon => {
                                let value = match Expression::parse(&mut self.tokenizer) {
                                    Ok(e) => e,
                                    Err(e) => return Some(Err(e)),
                                };

                                let child = match holding_pen.pop() {
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
                                                return Some(Err(SassError {
                                                    offset: name_lexeme.offset.unwrap_or(0),
                                                    kind: ErrorKind::ParserError,
                                                    message: format!(
                                                        "Expected to have seen a property or variable name, instead saw {:?}", other
                                                    ),
                                                }))
                                            },
                                        }
                                    },
                                    None => {
                                        return Some(Err(SassError {
                                            offset: lexeme.offset.unwrap_or(0),
                                            kind: ErrorKind::ParserError,
                                            message: format!(
                                                "Expected to have seen a property or variable name, did not see any"
                                            ),
                                        }))
                                    },
                                };
                                current_sass_rule.children.push(child);
                            },
                            Token::String(_) => {
                                holding_pen.push(lexeme);
                            },
                            Token::Comment(_) => {
                                current_sass_rule.children.push(
                                    Node::Comment(
                                        SassComment { content: lexeme }
                                    )
                                );
                            },
                            other => unreachable!("What is this??? {:?}", other),
                        }
                    }
                    return Some(Ok(Root::Rule(current_sass_rule)))
                },
                Token::Comma => {
                    current_sass_rule.selectors.push(selector_holding_pen);
                    selector_holding_pen = Lexeme::new();
                },
                Token::Comment(_) => {
                    return Some(Ok(Root::Comment(SassComment { content: lexeme })))
                },
                _ => {
                    selector_holding_pen = selector_holding_pen.combine(&lexeme);
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
    use token::{Token, Lexeme};

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
                selectors: vec![Lexeme { token: Token::String("a".into()), offset: Some(0) }],
                children: vec![Node::Property(
                    Lexeme { token: Token::String("color".into()), offset: Some(4) },
                    Expression::String(
                        Lexeme { token: Token::String("blue".into()), offset: Some(11) }
                    ),
                )],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_nested_rules() {
        let mut parser = Parser::new("div { img { color: blue; } }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![Lexeme { token: Token::String("div".into()), offset: Some(0) }],
                children: vec![Node::Rule(
                    SassRule {
                        selectors: vec![Lexeme {
                            token: Token::String("img".into()),
                            offset: Some(6)
                        }],
                        children: vec![Node::Property(
                            Lexeme { token: Token::String("color".into()), offset: Some(12) },
                            Expression::String(
                                Lexeme { token: Token::String("blue".into()), offset: Some(19) }
                            ),
                        )],
                    }
                )],
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
                    Lexeme { token: Token::String("a".into()), offset: Some(0) },
                    Lexeme { token: Token::String("b c".into()), offset: Some(3) },
                ],
                children: vec![Node::Property(
                    Lexeme { token: Token::String("color".into()), offset: Some(9) },
                    Expression::String(
                        Lexeme { token: Token::String("red".into()), offset: Some(16) }
                    ),
                )],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_variable_setting_statements() {
        let mut parser = Parser::new("$color: red;");
        assert_eq!(parser.next(), Some(Ok(Root::Variable(SassVariable {
            name: Lexeme { token: Token::String("$color".into()), offset: Some(0) },
            value: Expression::String(
                Lexeme { token: Token::String("red".into()), offset: Some(8) }
            ),
        }))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_variables_set_within_rules() {
        let mut parser = Parser::new("a { $foo: red; }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![
                    Lexeme { token: Token::String("a".into()), offset: Some(0) },
                ],
                children: vec![Node::Variable(SassVariable {
                    name: Lexeme { token: Token::String("$foo".into()), offset: Some(4) },
                    value: Expression::String(
                        Lexeme { token: Token::String("red".into()), offset: Some(10) }
                    ),
                })],
            }
        ))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_a_comment() {
        let mut parser = Parser::new("/* hi */");
        assert_eq!(parser.next(), Some(Ok(Root::Comment(SassComment {
            content: Lexeme { token: Token::Comment("/* hi */".into()), offset: Some(0) },
        }))));
        assert_eq!(parser.next(), None);
    }
}
