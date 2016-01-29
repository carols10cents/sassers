use token::{Token, Lexeme};
use tokenizer::Tokenizer;
use sass::rule::SassRule;
use sass::output_style::SassOutputStyle;
use error::{Result};

use std::io::Write;

pub struct Parser<'a> {
    pub tokenizer: Tokenizer<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTRoot {
    Rule(SassRule),
}

impl ASTRoot {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            ASTRoot::Rule(ref sr) => sr.stream(output, style),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Rule(SassRule),
    Property(Lexeme, Lexeme),
}

impl ASTNode {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            ASTNode::Rule(ref sr) => try!(sr.stream(output, style)),
            ASTNode::Property(ref name, ref value) => {
                let ref n = name.token;
                let ref v = value.token;
                // grumble mumble format strings you know they're a string literal
                let property = match style {
                    SassOutputStyle::Nested     => format!("  {}: {};", n, v),
                    SassOutputStyle::Compressed => format!("{}:{}", n, v),
                    SassOutputStyle::Expanded   => format!("  {}: {};", n, v),
                    SassOutputStyle::Compact    => format!("{}: {};", n, v),
                    SassOutputStyle::Debug      => format!("{:?}\n", self),
                    _ => unreachable!(),
                };
                try!(write!(output, "{}", property));
            },
        }
        Ok(())
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<ASTRoot>;

    fn next(&mut self) -> Option<Result<ASTRoot>> {
        let mut current_sass_rule = SassRule::new();
        let mut rule_stack = vec![];
        while let Some(Ok(lexeme)) = self.tokenizer.next() {
            if lexeme.token == Token::LeftCurlyBrace {
                let mut holding_pen = vec![];
                while let Some(Ok(lexeme)) = self.tokenizer.next() {
                    if lexeme.token == Token::RightCurlyBrace {
                        if rule_stack.is_empty() {
                            return Some(Ok(ASTRoot::Rule(current_sass_rule)))
                        } else {
                            let tmp_rule = current_sass_rule;
                            current_sass_rule = rule_stack.pop().unwrap();
                            current_sass_rule.children.push(ASTNode::Rule(tmp_rule));
                        }
                    } else if lexeme.token == Token::LeftCurlyBrace {
                        rule_stack.push(current_sass_rule);
                        current_sass_rule = SassRule::new();
                        current_sass_rule.selectors = holding_pen;
                        holding_pen = vec![];
                    } else if lexeme.token == Token::Colon {
                        // TODO: these unwraps had better work
                        let property_value = self.tokenizer.next().unwrap().unwrap();
                        // TODO: this had better be a semicolon
                        self.tokenizer.next();
                        // TODO: holding pen better have exactly one thing in it
                        let child = ASTNode::Property(holding_pen.pop().unwrap(), property_value);
                        current_sass_rule.children.push(child);
                    } else {
                        holding_pen.push(lexeme);
                    }
                }
                return Some(Ok(ASTRoot::Rule(current_sass_rule)))
            } else {
                current_sass_rule.selectors.push(lexeme);
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
    use token::{Token, Lexeme};

    #[test]
    fn it_returns_none_for_empty_string() {
        let mut parser = Parser::new("");
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_returns_a_rule() {
        let mut parser = Parser::new("a { color: blue; }");
        assert_eq!(parser.next(), Some(Ok(ASTRoot::Rule(
            SassRule {
                selectors: vec![Lexeme { token: Token::Ident("a".into()), offset: Some(0) }],
                children: vec![ASTNode::Property(
                    Lexeme { token: Token::Ident("color".into()), offset: Some(4) },
                    Lexeme { token: Token::Ident("blue".into()), offset: Some(11) },
                )],
            }
        ))));
    }

    #[test]
    fn it_returns_nested_rules() {
        let mut parser = Parser::new("div { img { color: blue; } }");
        assert_eq!(parser.next(), Some(Ok(ASTRoot::Rule(
            SassRule {
                selectors: vec![Lexeme { token: Token::Ident("div".into()), offset: Some(0) }],
                children: vec![ASTNode::Rule(
                    SassRule {
                        selectors: vec![Lexeme {
                            token: Token::Ident("img".into()),
                            offset: Some(6)
                        }],
                        children: vec![ASTNode::Property(
                            Lexeme { token: Token::Ident("color".into()), offset: Some(12) },
                            Lexeme { token: Token::Ident("blue".into()), offset: Some(19) },
                        )],
                    }
                )],
            }
        ))));
    }
}
