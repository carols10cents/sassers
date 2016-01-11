use token::Token;
use error::Result;

use std::str::CharIndices;

pub struct Tokenizer<'a> {
    text: &'a str,
    chars: CharIndices<'a>,
    offset: usize,
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
            chars: text.char_indices(),
            offset: 0,
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
                // First non-whitespace - save its offset to be the Token's offset
                value.push(curr_char);
                start = char_offset;
                while let Some((char_offset, curr_char)) = self.chars.next() {
                    // Stop when we reach whitespace
                    if curr_char.is_whitespace() {
                        break;
                    } else {
                        value.push(curr_char);
                    }
                }
            }
            return Ok(Some(Token { value: value, offset: Some(start) }))
        }
        return Ok(None)
    }
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
        let mut tokenizer = Tokenizer::new("}a{");
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("}", 0))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("a", 1))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::new("{", 2))));
        assert_eq!(tokenizer.next(), None);
    }
}
