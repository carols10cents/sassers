use token::Token;
use error::Result;

use std::str::CharIndices;
use std::iter::Peekable;

pub struct Tokenizer<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
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

    pub fn parse(&mut self) -> Result<Option<Token>> {
        let mut value = String::new();
        let mut start;
        while let Some((char_offset, curr_char)) = self.chars.next() {
            // Skip leading whitespace
            if curr_char.is_whitespace() {
                continue;
            } else {
                if is_single_char_token(curr_char) {
                    return Ok(Some(Token {
                        value: curr_char.to_string(),
                        offset: Some(char_offset),
                    }))
                } else {
                    // Start of a multi-char token
                    // First non-whitespace - save its offset to be the Token's offset
                    value.push(curr_char);
                    start = char_offset;
                    while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                        // Stop when we reach a non-word char
                        if next_char.is_whitespace() || is_single_char_token(next_char) {
                            break;
                        } else {
                            value.push(next_char);
                            self.chars.next();
                        }
                    }
                    return Ok(Some(Token { value: value, offset: Some(start) }))
                }
            }
        }
        return Ok(None)
    }
}

fn is_single_char_token(ch: char) -> bool {
    "{}:;".chars().any(|c| c == ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::Token;

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
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("div", 4))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("aoeu", 10))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_curly_braces() {
        // Without regard to matching
        let mut tokenizer = Tokenizer::new("{}}a{ blue}");
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("{", 0))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("}", 1))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("}", 2))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("a", 3))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("{", 4))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("blue", 6))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("}", 10))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_colon() {
        let mut tokenizer = Tokenizer::new(":invalid: property::");
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(":", 0))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("invalid", 1))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(":", 8))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("property", 10))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(":", 18))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(":", 19))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_semicolon() {
        let mut tokenizer = Tokenizer::new(";;\na;\nb\n;");
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(";", 0))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(";", 1))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("a", 2))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(";", 3))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("b", 4))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new(";", 5))));
        assert_eq!(tokenizer.next(), None);
    }
}
