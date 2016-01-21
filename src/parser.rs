use token::{Token, Lexeme};
use tokenizer::Tokenizer;
use sass::rule::SassRule;
use error::{Result};

pub struct Parser<'a> {
    pub tokenizer: Tokenizer<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTRoot {
    Rule(SassRule),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Property(Lexeme, Lexeme),
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<ASTRoot>;

    fn next(&mut self) -> Option<Result<ASTRoot>> {
        let mut current_sass_rule = SassRule::new();
        while let Some(Ok(lexeme)) = self.tokenizer.next() {
            if lexeme.token == Token::LeftCurlyBrace {
                while let Some(Ok(lexeme)) = self.tokenizer.next() {
                    if lexeme.token == Token::RightCurlyBrace {
                        return Some(Ok(ASTRoot::Rule(current_sass_rule)))
                    } else {
                        self.tokenizer.next(); // colon
                        let property_value = self.tokenizer.next().unwrap().unwrap();
                        self.tokenizer.next(); // semicolon

                        let child = ASTNode::Property(lexeme, property_value);
                        current_sass_rule.children.push(child);
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
                    Lexeme { token: Token::Ident("blue".into()), offset: Some(11) }
                )],
            }
        ))));
    }
}
