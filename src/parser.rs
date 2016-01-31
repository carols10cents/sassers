use token::{Token, Lexeme};
use tokenizer::Tokenizer;
use ast::expression::Expression;
use ast::root::Root;
use ast::node::Node;
use sass::rule::SassRule;
use error::{Result};

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
            if lexeme.token == Token::LeftCurlyBrace {
                current_sass_rule.selectors.push(selector_holding_pen);
                let mut holding_pen = vec![];
                while let Some(Ok(lexeme)) = self.tokenizer.next() {
                    if lexeme.token == Token::RightCurlyBrace {
                        if rule_stack.is_empty() {
                            return Some(Ok(Root::Rule(current_sass_rule)))
                        } else {
                            let tmp_rule = current_sass_rule;
                            current_sass_rule = rule_stack.pop().unwrap();
                            current_sass_rule.children.push(Node::Rule(tmp_rule));
                        }
                    } else if lexeme.token == Token::LeftCurlyBrace {
                        rule_stack.push(current_sass_rule);
                        current_sass_rule = SassRule::new();
                        current_sass_rule.selectors = holding_pen;
                        holding_pen = vec![];
                    } else if lexeme.token == Token::Colon {
                        let property_value = match Expression::parse(&mut self.tokenizer) {
                            Ok(e) => e,
                            Err(e) => return Some(Err(e)),
                        };

                        // TODO: holding pen better have exactly one thing in it
                        let child = Node::Property(holding_pen.pop().unwrap(), property_value);
                        current_sass_rule.children.push(child);
                    } else {
                        holding_pen.push(lexeme);
                    }
                }
                return Some(Ok(Root::Rule(current_sass_rule)))
            } else if lexeme.token == Token::Comma {
                current_sass_rule.selectors.push(selector_holding_pen);
                selector_holding_pen = Lexeme::new();
            } else {
                selector_holding_pen = selector_holding_pen.combine(&lexeme);
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
                selectors: vec![Lexeme { token: Token::Ident("a".into()), offset: Some(0) }],
                children: vec![Node::Property(
                    Lexeme { token: Token::Ident("color".into()), offset: Some(4) },
                    Expression::String(
                        Lexeme { token: Token::Ident("blue".into()), offset: Some(11) }
                    ),
                )],
            }
        ))));
    }

    #[test]
    fn it_returns_nested_rules() {
        let mut parser = Parser::new("div { img { color: blue; } }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![Lexeme { token: Token::Ident("div".into()), offset: Some(0) }],
                children: vec![Node::Rule(
                    SassRule {
                        selectors: vec![Lexeme {
                            token: Token::Ident("img".into()),
                            offset: Some(6)
                        }],
                        children: vec![Node::Property(
                            Lexeme { token: Token::Ident("color".into()), offset: Some(12) },
                            Expression::String(
                                Lexeme { token: Token::Ident("blue".into()), offset: Some(19) }
                            ),
                        )],
                    }
                )],
            }
        ))));
    }

    #[test]
    fn it_returns_rules_with_multiple_selectors() {
        let mut parser = Parser::new("a, b c { color: red; }");
        assert_eq!(parser.next(), Some(Ok(Root::Rule(
            SassRule {
                selectors: vec![
                    Lexeme { token: Token::Ident("a".into()), offset: Some(0) },
                    Lexeme { token: Token::Ident("b c".into()), offset: Some(3) },
                ],
                children: vec![Node::Property(
                    Lexeme { token: Token::Ident("color".into()), offset: Some(9) },
                    Expression::String(
                        Lexeme { token: Token::Ident("red".into()), offset: Some(16) }
                    ),
                )],
            }
        ))));
    }
}
