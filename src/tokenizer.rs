use token::{Token, TokenOffset};
use operator_or_token::OperatorOrToken;
use operator::Operator;
use operator_offset::OperatorOffset;
use error::{Result, SassError, ErrorKind};

use std::str::CharIndices;
use std::iter::Peekable;

pub struct Tokenizer<'a> {
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<OperatorOrToken>;

    fn next(&mut self) -> Option<Result<OperatorOrToken>> {
        match self.parse() {
            Ok(Some(t)) => Some(Ok(t)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &str) -> Tokenizer {
        Tokenizer {
            chars: text.char_indices().peekable(),
        }
    }

    // In the string parsing sense, not the parsing-out-semantics part yet.
    // Well, there's a tiny bit of semantics, depending on how you define "meaning".
    pub fn parse(&mut self) -> Result<Option<OperatorOrToken>> {
        while let Some((char_offset, curr_char)) = self.chars.next() {
            // Skip leading whitespace
            if curr_char.is_whitespace() {
                continue;
            } else {
                let single_char_op = Operator::from_char(curr_char);
                if single_char_op.is_some()
                   && !self.hyphen_starting_shit(curr_char)
                   && !self.multiline_comment_starting(curr_char)
                   && !self.singleline_comment_starting(curr_char) {
                    // We already tested that single_char_token was Some.
                    return Ok(Some(OperatorOrToken::Operator(OperatorOffset {
                        operator: single_char_op.unwrap(),
                        offset: Some(char_offset),
                    })))
                } else {
                    if self.multiline_comment_starting(curr_char) {
                        return self.multiline_comment(curr_char, char_offset)
                    } else if self.singleline_comment_starting(curr_char) {
                        self.discard_singleline_comment();
                        return self.parse()
                    } else if curr_char == '"' {
                        return self.string_literal(curr_char, char_offset)
                    } else if curr_char.is_numeric() || self.hyphen_starting_number(curr_char) {
                        return self.number(curr_char, char_offset)
                    } else {
                        return self.ident(curr_char, char_offset)
                    }
                }
            }
        }
        return Ok(None)
    }

    fn hyphen_starting_shit(&mut self, curr_char: char) -> bool {
        let peek_char = self.peek_char();
        curr_char == '-' && peek_char.is_some() && !peek_char.unwrap().is_whitespace()
    }

    fn multiline_comment_starting(&mut self, curr_char: char) -> bool {
        let peek_char = self.peek_char();
        curr_char == '/'
          && peek_char.is_some()
          && peek_char.unwrap() == '*'
    }

    fn singleline_comment_starting(&mut self, curr_char: char) -> bool {
        let peek_char = self.peek_char();
        curr_char == '/'
          && peek_char.is_some()
          && peek_char.unwrap() == '/'
    }

    fn hyphen_starting_number(&mut self, curr_char: char) -> bool {
        let peek_char = self.peek_char();
        curr_char == '-' && peek_char.is_some() && peek_char.unwrap().is_numeric()
    }

    fn peek_char(&mut self) -> Option<char> {
        match self.chars.peek() {
            Some(&(_, peek_char)) => Some(peek_char),
            None => None,
        }
    }

    fn ident(&mut self, curr_char: char, start: usize) -> Result<Option<OperatorOrToken>> {
        let mut value = String::new();
        value.push(curr_char);

        while let Some(peek_char) = self.peek_char() {
            // Stop when we reach a non-ident char (hyphens are special)
            if peek_char.is_whitespace() || (
                is_single_char_token(peek_char) && peek_char != '-'
            ) {
                break;
            } else {
                value.push(peek_char);
                self.chars.next();
            }
        }
        Ok(Some(OperatorOrToken::Token(
            TokenOffset { token: Token::String(value), offset: Some(start) }
        )))
    }

    fn number(&mut self, curr_char: char, start: usize) -> Result<Option<OperatorOrToken>> {
        let mut value = String::new();
        value.push(curr_char);

        while let Some(peek_char) = self.peek_char() {
            // Stop when we reach a non-numeric char
            // TODO: disallow two `.`s in one number
            if !peek_char.is_numeric() && peek_char != '.' {
                break;
            } else {
                value.push(peek_char);
                self.chars.next();
            }
        }

        let value = match value.parse() {
            Ok(v) => v,
            Err(_) => return Err(SassError {
                offset: start,
                kind: ErrorKind::TokenizerError,
                message: format!(
                    "Tried to parse `{}` into a f32 but failed.",
                    value,
                ),
            })
        };

        let mut unit = String::new();
        while let Some(peek_char) = self.peek_char() {
            // Get units; stop when we reach a space or non-percent operator
            if peek_char == '%' || (
                !peek_char.is_whitespace() && !is_single_char_token(peek_char)
            ) {
                unit.push(peek_char);
                self.chars.next();
            } else {
                break;
            }
        }

        let token = if unit.len() > 0 {
            Token::Number { value: value, units: Some(unit), computed: false }
        } else {
            Token::Number { value: value, units: None, computed: false }
        };

        Ok(Some(OperatorOrToken::Token(
            TokenOffset { token: token, offset: Some(start) }
        )))
    }

    fn string_literal(&mut self, curr_char: char, start: usize) -> Result<Option<OperatorOrToken>> {
        let mut value = String::new();
        value.push(curr_char);

        while let Some(peek_char) = self.peek_char() {
            if peek_char == '"' && !value.ends_with("\\") {
                value.push(peek_char);
                self.chars.next();
                break;
            } else {
                value.push(peek_char);
                self.chars.next();
            }
        }

        let token = Token::StringLiteral(value);

        Ok(Some(OperatorOrToken::Token(
            TokenOffset { token: token, offset: Some(start) }
        )))
    }

    fn multiline_comment(&mut self, curr_char: char, start: usize) -> Result<Option<OperatorOrToken>> {
        let mut value = String::new();
        value.push(curr_char);
        // We already tested that this was asterisk
        value.push(self.peek_char().unwrap());
        self.chars.next();

        while let Some(peek_char) = self.peek_char() {
            if peek_char == '/' && value.ends_with("*") {
                value.push(peek_char);
                self.chars.next();
                break;
            } else {
                value.push(peek_char);
                self.chars.next();
            }
        }

        let token = Token::Comment(value);

        Ok(Some(OperatorOrToken::Token(
            TokenOffset { token: token, offset: Some(start) }
        )))
    }

    fn discard_singleline_comment(&mut self) {
        while let Some(peek_char) = self.peek_char() {
            self.chars.next();
            if peek_char == '\n' {
                break;
            }
        }
    }
}

fn is_single_char_token(ch: char) -> bool {
    Operator::from_char(ch).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::{TokenOffset, Token};
    use operator_or_token::OperatorOrToken;
    use operator::Operator;
    use operator_offset::OperatorOffset;
    use error::Result;

    fn expected_token(expected_token: Token, expected_offset: usize) -> Option<Result<OperatorOrToken>> {
        Some(Ok(OperatorOrToken::Token(TokenOffset {
            token: expected_token,
            offset: Some(expected_offset),
        })))
    }

    fn expected_operator(expected_operator: Operator, expected_offset: usize) -> Option<Result<OperatorOrToken>> {
        Some(Ok(OperatorOrToken::Operator(OperatorOffset {
            operator: expected_operator,
            offset: Some(expected_offset),
        })))
    }

    fn expected_ident(expected_value: &str, expected_offset: usize) -> Option<Result<OperatorOrToken>> {
        expected_token(Token::String(expected_value.into()), expected_offset)
    }

    #[test]
    fn it_returns_none_for_empty_string() {
        let mut tokenizer = Tokenizer::new("");
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_returns_none_for_only_whitespace() {
        let mut tokenizer = Tokenizer::new("     ");
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_returns_words() {
        // Without regard to Sass word validity
        let mut tokenizer = Tokenizer::new(" \n  div   aoeu  ");
        assert_eq!(tokenizer.next(), expected_ident("div", 4));
        assert_eq!(tokenizer.next(), expected_ident("aoeu", 10));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_curly_braces() {
        // Without regard to matching
        let mut tokenizer = Tokenizer::new("{}}a{ blah}");
        assert_eq!(tokenizer.next(), expected_operator(
            Operator::LeftCurlyBrace, 0
        ));
        assert_eq!(tokenizer.next(), expected_operator(
            Operator::RightCurlyBrace, 1
        ));
        assert_eq!(tokenizer.next(), expected_operator(
            Operator::RightCurlyBrace, 2
        ));
        assert_eq!(tokenizer.next(), expected_ident("a", 3));
        assert_eq!(tokenizer.next(), expected_operator(
            Operator::LeftCurlyBrace, 4
        ));
        assert_eq!(tokenizer.next(), expected_ident("blah", 6));
        assert_eq!(tokenizer.next(), expected_operator(
            Operator::RightCurlyBrace, 10
        ));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_colon() {
        let mut tokenizer = Tokenizer::new(":invalid: property::");
        // This might be wrong. I have a note somewhere else that colons can start idents.
        assert_eq!(tokenizer.next(), expected_operator(Operator::Colon, 0));
        assert_eq!(tokenizer.next(), expected_ident("invalid", 1));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Colon, 8));
        assert_eq!(tokenizer.next(), expected_ident("property", 10));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Colon, 18));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Colon, 19));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_semicolon() {
        let mut tokenizer = Tokenizer::new(";;\na;\nb\n;");
        assert_eq!(tokenizer.next(), expected_operator(Operator::Semicolon, 0));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Semicolon, 1));
        assert_eq!(tokenizer.next(), expected_ident("a", 3));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Semicolon, 4));
        assert_eq!(tokenizer.next(), expected_ident("b", 6));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Semicolon, 8));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_numbers() {
        let mut tokenizer = Tokenizer::new("border: 0px 1.5 11em;");
        assert_eq!(tokenizer.next(), expected_ident("border", 0));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Colon, 6));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number {
                value: 0.0,
                units: Some("px".into()),
                computed: false,
            }, 8
        ));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number {
                value: 1.5,
                units: None,
                computed: false
            }, 12
        ));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number {
                value: 11.0,
                units: Some("em".into()),
                computed: false
            }, 16
        ));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Semicolon, 20));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_hyphen() {
        let mut tokenizer = Tokenizer::new("font-weight -webkit -3 - 4-5 a-1 -");
        assert_eq!(tokenizer.next(), expected_ident("font-weight", 0));
        assert_eq!(tokenizer.next(), expected_ident("-webkit", 12));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number { value: -3.0, units: None, computed: false }, 20
        ));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Minus, 23));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number { value: 4.0, units: None, computed: false }, 25
        ));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number { value: -5.0, units: None, computed: false }, 26
        ));
        assert_eq!(tokenizer.next(), expected_ident("a-1", 29));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Minus, 33));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_parens() {
        let mut tokenizer = Tokenizer::new("() rgb)()(");
        assert_eq!(tokenizer.next(), expected_operator(Operator::LeftParen, 0));
        assert_eq!(tokenizer.next(), expected_operator(Operator::RightParen, 1));
        assert_eq!(tokenizer.next(), expected_ident("rgb", 3));
        assert_eq!(tokenizer.next(), expected_operator(Operator::RightParen, 6));
        assert_eq!(tokenizer.next(), expected_operator(Operator::LeftParen, 7));
        assert_eq!(tokenizer.next(), expected_operator(Operator::RightParen, 8));
        assert_eq!(tokenizer.next(), expected_operator(Operator::LeftParen, 9));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_slash() {
        let mut tokenizer = Tokenizer::new("/ / 3/4 / 8");
        assert_eq!(tokenizer.next(), expected_operator(Operator::Slash, 0));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Slash, 2));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number { value: 3.0, units: None, computed: false }, 4
        ));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Slash, 5));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number { value: 4.0, units: None, computed: false }, 6
        ));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Slash, 8));
        assert_eq!(tokenizer.next(), expected_token(
            Token::Number { value: 8.0, units: None, computed: false }, 10
        ));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_comma() {
        let mut tokenizer = Tokenizer::new("a, b {");
        assert_eq!(tokenizer.next(), expected_ident("a", 0));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Comma, 1));
        assert_eq!(tokenizer.next(), expected_ident("b", 3));
        assert_eq!(tokenizer.next(), expected_operator(Operator::LeftCurlyBrace, 5));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_double_quoted_string_literals() {
        let mut tokenizer = Tokenizer::new("what: \"hey \\\"ya\";");
        assert_eq!(tokenizer.next(), expected_ident("what", 0));
        assert_eq!(tokenizer.next(), expected_operator(Operator::Colon, 4));
        assert_eq!(
            tokenizer.next(),
            expected_token(Token::StringLiteral("\"hey \\\"ya\"".into()), 6)
        );
        assert_eq!(tokenizer.next(), expected_operator(Operator::Semicolon, 16));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_multiline_comments() {
        let mut tokenizer = Tokenizer::new("a /* foo\nbar */ no");
        assert_eq!(tokenizer.next(), expected_ident("a", 0));
        assert_eq!(tokenizer.next(), expected_token(Token::Comment("/* foo\nbar */".into()), 2));
        assert_eq!(tokenizer.next(), expected_ident("no", 16));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_removes_single_line_comments() {
        let mut tokenizer = Tokenizer::new("b // because; hi \nc");
        assert_eq!(tokenizer.next(), expected_ident("b", 0));
        assert_eq!(tokenizer.next(), expected_ident("c", 18));
        assert_eq!(tokenizer.next(), None);
    }
}
