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
                } else if curr_char == '-' {
                    value.push(curr_char);
                    start = char_offset;

                    if let Some(&(_, peek_char)) = self.chars.peek() {
                        if peek_char.is_numeric() {
                            while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                                // Stop when we reach a non-numeric char
                                if !next_char.is_numeric() && next_char != '.' {
                                    break;
                                } else {
                                    value.push(next_char);
                                    self.chars.next();
                                }
                            }
                        } else {
                            while let Some(&(next_char_offset, next_char)) = self.chars.peek() {
                                // Stop when we reach a non-word char
                                if next_char.is_whitespace() || is_single_char_token(next_char) {
                                    break;
                                } else {
                                    value.push(next_char);
                                    self.chars.next();
                                }
                            }
                        }
                    }

                    return Ok(Some(Token { value: value, offset: Some(start) }))
                } else if curr_char.is_numeric() {
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
                    return Ok(Some(Token { value: value, offset: Some(start) }))
                } else {
                    // Start of a multi-char non-numeric token
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
    "{}():;/".chars().any(|c| c == ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::Token;
    use error::Result;

    fn assert_expected_token(
        actual: Option<Result<Token>>,
        expected_value: &str,
        expected_offset: usize) {
        let actual = actual.unwrap().unwrap();
        assert_eq!(actual.value, expected_value);
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
        assert_expected_token(tokenizer.next(), "div", 4);
        assert_expected_token(tokenizer.next(), "aoeu", 10);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_curly_braces() {
        // Without regard to matching
        let mut tokenizer = Tokenizer::new("{}}a{ blue}");
        assert_expected_token(tokenizer.next(), "{", 0);
        assert_expected_token(tokenizer.next(), "}", 1);
        assert_expected_token(tokenizer.next(), "}", 2);
        assert_expected_token(tokenizer.next(), "a", 3);
        assert_expected_token(tokenizer.next(), "{", 4);
        assert_expected_token(tokenizer.next(), "blue", 6);
        assert_expected_token(tokenizer.next(), "}", 10);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_colon() {
        let mut tokenizer = Tokenizer::new(":invalid: property::");
        assert_expected_token(tokenizer.next(), ":", 0);
        assert_expected_token(tokenizer.next(), "invalid", 1);
        assert_expected_token(tokenizer.next(), ":", 8);
        assert_expected_token(tokenizer.next(), "property", 10);
        assert_expected_token(tokenizer.next(), ":", 18);
        assert_expected_token(tokenizer.next(), ":", 19);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_semicolon() {
        let mut tokenizer = Tokenizer::new(";;\na;\nb\n;");
        assert_expected_token(tokenizer.next(), ";", 0);
        assert_expected_token(tokenizer.next(), ";", 1);
        assert_expected_token(tokenizer.next(), "a", 3);
        assert_expected_token(tokenizer.next(), ";", 4);
        assert_expected_token(tokenizer.next(), "b", 6);
        assert_expected_token(tokenizer.next(), ";", 8);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_numbers() {
        let mut tokenizer = Tokenizer::new("border: 0px 1.5 11em;");
        assert_expected_token(tokenizer.next(), "border", 0);
        assert_expected_token(tokenizer.next(), ":", 6);
        assert_expected_token(tokenizer.next(), "0", 8);
        assert_expected_token(tokenizer.next(), "px", 9);
        assert_expected_token(tokenizer.next(), "1.5", 12);
        assert_expected_token(tokenizer.next(), "11", 16);
        assert_expected_token(tokenizer.next(), "em", 18);
        assert_expected_token(tokenizer.next(), ";", 20);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_hyphen() {
        let mut tokenizer = Tokenizer::new("font-weight -webkit -3 - 4-5 a-1");
        assert_expected_token(tokenizer.next(), "font-weight", 0);
        assert_expected_token(tokenizer.next(), "-webkit", 12);
        assert_expected_token(tokenizer.next(), "-3", 20);
        assert_expected_token(tokenizer.next(), "-", 23);
        assert_expected_token(tokenizer.next(), "4", 25);
        assert_expected_token(tokenizer.next(), "-5", 26);
        assert_expected_token(tokenizer.next(), "a-1", 29);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_parens() {
        let mut tokenizer = Tokenizer::new("() rgb)()(");
        assert_expected_token(tokenizer.next(), "(", 0);
        assert_expected_token(tokenizer.next(), ")", 1);
        assert_expected_token(tokenizer.next(), "rgb", 3);
        assert_expected_token(tokenizer.next(), ")", 6);
        assert_expected_token(tokenizer.next(), "(", 7);
        assert_expected_token(tokenizer.next(), ")", 8);
        assert_expected_token(tokenizer.next(), "(", 9);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn it_separates_slash() {
        let mut tokenizer = Tokenizer::new("/ / 3/4 / 8");
        assert_expected_token(tokenizer.next(), "/", 0);
        assert_expected_token(tokenizer.next(), "/", 2);
        assert_expected_token(tokenizer.next(), "3", 4);
        assert_expected_token(tokenizer.next(), "/", 5);
        assert_expected_token(tokenizer.next(), "4", 6);
        assert_expected_token(tokenizer.next(), "/", 8);
        assert_expected_token(tokenizer.next(), "8", 10);
        assert_eq!(tokenizer.next(), None);
    }
}
