use token::{Lexeme, Token};
use error::{Result, SassError, ErrorKind};

use std::str::CharIndices;
use std::iter::Peekable;

pub struct Tokenizer<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Lexeme>;

    fn next(&mut self) -> Option<Result<Lexeme>> {
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
            text: text,
            chars: text.char_indices().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Option<Lexeme>> {
        let mut value = String::new();
        let mut start;

        while let Some((char_offset, curr_char)) = self.chars.next() {
            // Skip leading whitespace
            if curr_char.is_whitespace() {
                continue;
            } else {
                let single_char_token = Token::from_char(curr_char);
                if value.is_empty() && single_char_token.is_some() {
                    if curr_char == '-' {
                        value.push(curr_char);
                        start = char_offset;

                        if let Some(&(_, peek_char)) = self.chars.peek() {
                            if peek_char.is_whitespace() {
                                return Ok(Some(Lexeme { token: Token::Minus, offset: Some(start) }))
                            } else if peek_char.is_numeric() {
                                while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                                    // Stop when we reach a non-numeric char
                                    if !next_char.is_numeric() && next_char != '.' {
                                        break;
                                    } else {
                                        value.push(next_char);
                                        self.chars.next();
                                    }
                                }

                                let value = match value.parse() {
                                    Ok(v) => v,
                                    Err(e) => return Err(SassError {
                                        offset: start,
                                        kind: ErrorKind::TokenizerError,
                                        message: format!(
                                            "Tried to parse `{}` into a f32 but failed.",
                                            value,
                                        ),
                                    })
                                };
                                return Ok(Some(Lexeme { token: Token::Number(value), offset: Some(start) }))

                            } else {
                                while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                                    // Stop when we reach a non-ident char (hyphens are special)
                                    if next_char.is_whitespace() || (
                                        is_single_char_token(next_char) && next_char != '-'
                                    ) {
                                        break;
                                    } else {
                                        value.push(next_char);
                                        self.chars.next();
                                    }
                                }
                                return Ok(Some(Lexeme { token: Token::Ident(value), offset: Some(start) }))
                            }
                        }
                    } else {
                        // We already tested that single_char_token was Some.
                        return Ok(Some(Lexeme {
                            token: single_char_token.unwrap(),
                            offset: Some(char_offset),
                        }))
                    }
                } else {

                    if curr_char.is_numeric() {
                        // Start of a multi-char numeric token
                        value.push(curr_char);
                        start = char_offset;

                        while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                            // Stop when we reach a non-numeric char
                            if !next_char.is_numeric() && next_char != '.' {
                                break;
                            } else {
                                value.push(next_char);
                                self.chars.next();
                            }
                        }
                        let value = match value.parse() {
                            Ok(v) => v,
                            Err(e) => return Err(SassError {
                                offset: start,
                                kind: ErrorKind::TokenizerError,
                                message: format!(
                                    "Tried to parse `{}` into a f32 but failed.",
                                    value,
                                ),
                            })
                        };
                        return Ok(Some(Lexeme { token: Token::Number(value), offset: Some(start) }))
                    } else {
                        // Start of a multi-char non-numeric token
                        value.push(curr_char);
                        start = char_offset;
                        while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                            // Stop when we reach a non-ident char (hyphens are special)
                            if next_char.is_whitespace() || (
                                is_single_char_token(next_char) && next_char != '-'
                            ) {
                                break;
                            } else {
                                value.push(next_char);
                                self.chars.next();
                            }
                        }
                        return Ok(Some(Lexeme { token: Token::Ident(value), offset: Some(start) }))
                    }
                }
            }
        }
        return Ok(None)
    }
}

fn is_single_char_token(ch: char) -> bool {
    Token::from_char(ch).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::{Lexeme, Token};
    use error::Result;

    fn assert_expected_lexeme(
        actual: Option<Result<Lexeme>>,
        expected_token: Token,
        expected_offset: usize) {
        let actual = actual.unwrap().unwrap();
        assert_eq!(actual.token, expected_token);
        assert_eq!(actual.offset, Some(expected_offset));
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
        assert_expected_lexeme(tokenizer.next(), Token::Ident("div".into()), 4);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("aoeu".into()), 10);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_curly_braces() {
        // Without regard to matching
        let mut tokenizer = Tokenizer::new("{}}a{ blah}");
        assert_expected_lexeme(tokenizer.next(), Token::LeftCurlyBrace, 0);
        assert_expected_lexeme(tokenizer.next(), Token::RightCurlyBrace, 1);
        assert_expected_lexeme(tokenizer.next(), Token::RightCurlyBrace, 2);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("a".into()), 3);
        assert_expected_lexeme(tokenizer.next(), Token::LeftCurlyBrace, 4);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("blah".into()), 6);
        assert_expected_lexeme(tokenizer.next(), Token::RightCurlyBrace, 10);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_colon() {
        let mut tokenizer = Tokenizer::new(":invalid: property::");
        // This might be wrong. I have a note somewhere else that colons can start idents.
        assert_expected_lexeme(tokenizer.next(), Token::Colon, 0);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("invalid".into()), 1);
        assert_expected_lexeme(tokenizer.next(), Token::Colon, 8);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("property".into()), 10);
        assert_expected_lexeme(tokenizer.next(), Token::Colon, 18);
        assert_expected_lexeme(tokenizer.next(), Token::Colon, 19);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_semicolon() {
        let mut tokenizer = Tokenizer::new(";;\na;\nb\n;");
        assert_expected_lexeme(tokenizer.next(), Token::Semicolon, 0);
        assert_expected_lexeme(tokenizer.next(), Token::Semicolon, 1);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("a".into()), 3);
        assert_expected_lexeme(tokenizer.next(), Token::Semicolon, 4);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("b".into()), 6);
        assert_expected_lexeme(tokenizer.next(), Token::Semicolon, 8);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_numbers() {
        let mut tokenizer = Tokenizer::new("border: 0px 1.5 11em;");
        assert_expected_lexeme(tokenizer.next(), Token::Ident("border".into()), 0);
        assert_expected_lexeme(tokenizer.next(), Token::Colon, 6);
        assert_expected_lexeme(tokenizer.next(), Token::Number(0.0), 8);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("px".into()), 9);
        assert_expected_lexeme(tokenizer.next(), Token::Number(1.5), 12);
        assert_expected_lexeme(tokenizer.next(), Token::Number(11.0), 16);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("em".into()), 18);
        assert_expected_lexeme(tokenizer.next(), Token::Semicolon, 20);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_hyphen() {
        let mut tokenizer = Tokenizer::new("font-weight -webkit -3 - 4-5 a-1");
        assert_expected_lexeme(tokenizer.next(), Token::Ident("font-weight".into()), 0);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("-webkit".into()), 12);
        assert_expected_lexeme(tokenizer.next(), Token::Number(-3.0), 20);
        assert_expected_lexeme(tokenizer.next(), Token::Minus, 23);
        assert_expected_lexeme(tokenizer.next(), Token::Number(4.0), 25);
        assert_expected_lexeme(tokenizer.next(), Token::Number(-5.0), 26);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("a-1".into()), 29);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_parens() {
        let mut tokenizer = Tokenizer::new("() rgb)()(");
        assert_expected_lexeme(tokenizer.next(), Token::LeftParen, 0);
        assert_expected_lexeme(tokenizer.next(), Token::RightParen, 1);
        assert_expected_lexeme(tokenizer.next(), Token::Ident("rgb".into()), 3);
        assert_expected_lexeme(tokenizer.next(), Token::RightParen, 6);
        assert_expected_lexeme(tokenizer.next(), Token::LeftParen, 7);
        assert_expected_lexeme(tokenizer.next(), Token::RightParen, 8);
        assert_expected_lexeme(tokenizer.next(), Token::LeftParen, 9);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_slash() {
        let mut tokenizer = Tokenizer::new("/ / 3/4 / 8");
        assert_expected_lexeme(tokenizer.next(), Token::Slash, 0);
        assert_expected_lexeme(tokenizer.next(), Token::Slash, 2);
        assert_expected_lexeme(tokenizer.next(), Token::Number(3.0), 4);
        assert_expected_lexeme(tokenizer.next(), Token::Slash, 5);
        assert_expected_lexeme(tokenizer.next(), Token::Number(4.0), 6);
        assert_expected_lexeme(tokenizer.next(), Token::Slash, 8);
        assert_expected_lexeme(tokenizer.next(), Token::Number(8.0), 10);
        assert_eq!(tokenizer.next(), None);
    }
}
