use error::{SassError, ErrorKind, Result};
use event::Event;
use sass::mixin::{SassMixin, SassMixinCall};
use sass::parameters::{SassParameter, SassArgument};
use inner_tokenizer::{InnerTokenizer, State};

use std::cmp;

pub fn is_space(c: u8) -> bool {
    c == b' '
}

pub fn isnt_space(c: u8) -> bool {
    !is_space(c)
}

pub fn is_ascii_whitespace(c: u8) -> bool {
    is_newline(c) || is_ascii_whitespace_no_nl(c)
}

pub fn is_ascii_whitespace_no_nl(c: u8) -> bool {
    c == b'\t' || c == 0x0b || c == 0x0c || c == b' '
}

pub fn is_newline(c: u8) -> bool {
    c == b'\n' || c == b'\r'
}

pub fn isnt_newline(c: u8) -> bool {
    !is_newline(c)
}

pub fn valid_unit_char(c: u8) -> bool {
    c == b'%' || (!is_space(c) && !is_operator(c))
}

pub fn valid_hex_char(c: u8) -> bool {
    match c {
        b'0' ... b'9' | b'a' ... b'f' => true,
        _ => false,
    }
}

pub fn valid_selector_char(c: u8) -> bool {
    c != b',' && c != b'{' && c != b':'
}

pub fn valid_name_char(c: u8) -> bool {
    c != b':' && c != b'{' && c != b'(' && c != b')' && c != b';'
}

pub fn valid_string_char(c: u8) -> bool {
    c != b'(' && isnt_space(c)
}

pub fn valid_mixin_arg_char(c: u8) -> bool {
    c != b',' && c != b')'
}

pub fn is_number(c: u8) -> bool {
    let result = match c {
        b'0' ... b'9' | b'.' => true,
        _ => false,
    };
    result
}

pub fn isnt_asterisk(c: u8) -> bool {
    c != b'*'
}

pub fn isnt_semicolon(c: u8) -> bool {
    c != b';'
}

pub fn is_operator(c: u8) -> bool {
    match c {
        b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')' | b',' => true,
        _ => false,
    }
}

// unusual among "scan" functions in that it scans from the _back_ of the string
// TODO: should also scan unicode whitespace?
pub fn scan_trailing_whitespace(data: &str) -> usize {
    match data.as_bytes().iter().rev().position(|&c| !is_ascii_whitespace_no_nl(c)) {
        Some(i) => i,
        None => data.len()
    }
}

#[derive(Debug, Clone)]
pub struct Toker<'a> {
    pub inner_str: &'a str,
    pub offset: usize,
}

impl<'a> Toker<'a> {
    pub fn limit(&self) -> usize {
        self.inner_str.len()
    }

    pub fn bytes(&self) -> &[u8] {
        self.inner_str.as_bytes()
    }

    pub fn at_eof(&self) -> bool {
        self.offset == self.limit()
    }

    pub fn curr_byte(&self) -> u8 {
        self.bytes()[self.offset]
    }

    pub fn next_byte(&self) -> u8 {
        self.bytes()[self.offset + 1]
    }

    pub fn eat(&mut self, expected: &str) -> Result<bool> {
        let original_offset = self.offset;
        for c in expected.as_bytes().iter() {
            match self.eatch(c) {
                Ok(true) => {},
                Ok(false) => {
                    let error_offset = self.offset;
                    self.offset = original_offset;
                    return Err(SassError {
                        offset: error_offset,
                        kind: ErrorKind::TokenizerError,
                        message: format!(
                            "Expected: `{}`, Saw: `{}`",
                            expected,
                            String::from_utf8_lossy(&self.bytes()[
                                self.offset..cmp::min(self.offset + expected.len(), self.limit())
                            ])
                        ),
                    })
                }
                Err(e) => {
                    return Err(e)
                },
            }
        }
        Ok(true)
    }

    fn eatch(&mut self, expected_char: &u8) -> Result<bool> {
        if self.at_eof() {
            let error_offset = self.offset;
            Err(SassError {
                offset: error_offset,
                kind: ErrorKind::UnexpectedEof,
                message: format!(
                    "Expected: `{}`; reached EOF instead.",
                    *expected_char as char
                ),
            })
        } else {
            if self.bytes()[self.offset] == *expected_char {
                self.offset += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    pub fn scan_while_or_end<F>(&self, start: usize, f: F) -> usize
            where F: Fn(u8) -> bool {
        let end = self.limit();
        self.scan_while(&self.inner_str[start..end], f)
    }

    fn scan_while<F>(&self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
        }
    }

    pub fn skip_leading_whitespace(&mut self) {
        let mut i = self.offset;

        while i < self.limit() {
            let c = self.bytes()[i];
            if is_ascii_whitespace(c) {
                i += self.scan_while_or_end(i, is_ascii_whitespace);
            } else if c == b'/' && i + 1 < self.limit() && self.bytes()[i + 1] == b'/' {
                i += self.scan_while_or_end(i, isnt_newline);
            } else {
                self.offset = i;
                return
            }
        }
        self.offset = self.limit();
    }

    pub fn next_name(&mut self) -> Result<String> {
        debug!("next_name, offset {:?}", self.offset);
        let name_beginning = self.offset;
        let mut i = name_beginning;

        // Colons are valid at the beginning of a name
        if self.eat(":").is_ok() {
            i = self.offset;
        }

        while i < self.limit() {
            i += self.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            self.offset = i;
            return Ok(String::from(&self.inner_str[name_beginning..name_end]))
        }
        let error_offset = self.offset;
        self.offset = self.limit();
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected a valid name; reached EOF instead."
            ),
        })
    }

    pub fn next_value(&mut self) -> Result<String> {
        debug!("next_name, offset {:?}", self.offset);
        let value_beginning = self.offset;
        let mut i = value_beginning;

        while i < self.limit() {
            i += self.scan_while_or_end(i, isnt_semicolon);
            let value_end = i;
            self.offset = i;
            return Ok(String::from(&self.inner_str[value_beginning..value_end]))
        }
        let error_offset = self.offset;
        self.offset = self.limit();
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected a valid value; reached EOF instead."
            ),
        })
    }

    pub fn tokenize_list<F>(&mut self, separator: &str, end_list: &str, valid_char_fn: &F) -> Result<Vec<String>>
        where F: Fn(u8) -> bool {
        let mut list = Vec::new();

        let mut i = self.offset;
        while i < self.limit() {
            self.skip_leading_whitespace();
            i = self.offset;
            let beginning = self.offset;
            i += self.scan_while_or_end(i, valid_char_fn);

            let n = scan_trailing_whitespace(&self.inner_str[beginning..i]);
            let end = i - n;

            if end > beginning {
                list.push(String::from(&self.inner_str[beginning..end]));
            }

            self.offset = i;
            if self.eat(end_list).is_ok() {
                break;
            } else {
                match self.eat(separator) {
                    Err(e) => {
                        return Err(SassError {
                            message: format!("While tokenizing list with separator `{}` and end list `{}`:\n{}", separator, end_list, e.message),
                            ..e
                        })
                    }
                    _ => {},
                }
            }
        }

        Ok(list)
    }

    pub fn next_mixin(&mut self) -> Result<Option<Event>> {
        debug!("next_mixin, offset {:?}", self.offset);
        let name_beginning = self.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.scan_while_or_end(i, valid_name_char);
            let name_end = i;

            self.offset = i;
            try!(self.eat("("));

            let parameters = try!(self.tokenize_list(",", ")", &valid_mixin_arg_char));

            self.skip_leading_whitespace();
            try!(self.eat("{"));
            self.skip_leading_whitespace();

            let mut children = Vec::new();
            let mut inner = InnerTokenizer {
                toker: self.clone(),
                state: State::InProperties,
            };
            while let Some(Ok(e)) = inner.next() {
                children.push(e);
            }
            self.offset = inner.toker.offset;

            try!(self.eat("}"));

            let mixin = Event::Mixin(SassMixin {
                name: String::from(&self.inner_str[name_beginning..name_end]),
                parameters: parameters.into_iter().map(|a|
                    SassParameter::new(a)
                ).collect(),
                children: children,
            });

            return Ok(Some(mixin))
        }
        let error_offset = self.offset;
        self.offset = self.limit();
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected mixin declaration; reached EOF instead."
            ),
        })
    }

    pub fn next_mixin_call(&mut self) -> Result<Option<Event>> {
        debug!("next_mixin_call, offset {:?}", self.offset);
        self.skip_leading_whitespace();
        let name_beginning = self.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            let name = String::from(&self.inner_str[name_beginning..name_end]);

            self.offset = i;

            let arguments = if self.eat("(").is_ok() {
                try!(self.tokenize_list(",", ")", &valid_mixin_arg_char))
            } else {
                Vec::new()
            };

            try!(self.eat(";"));

            let mixin_call = Event::MixinCall(SassMixinCall {
                name: name,
                arguments: arguments.into_iter().map(|a|
                    SassArgument::new(a)
                ).collect(),
            });

            return Ok(Some(mixin_call))

        }
        let error_offset = self.offset;
        self.offset = self.limit();
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected mixin call; reached EOF instead."
            ),
        })
    }

    pub fn next_comment(&mut self) -> Result<Option<Event>> {
        debug!("next_comment, offset {:?}", self.offset);
        let comment_body_beginning = self.offset;
        let mut i = comment_body_beginning + 2;

        while i < self.limit() {
            i += self.scan_while_or_end(i, isnt_asterisk);
            self.offset = i;

            if self.eat("*/").is_ok() {
                return Ok(Some(
                    Event::Comment(String::from(
                        &self.inner_str[comment_body_beginning..self.offset]
                    ))
                ))
            } else {
                i += 1;
            }
        }
        let error_offset = self.offset;
        self.offset = self.limit();
        Err(SassError {
            offset: error_offset,
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected comment; reached EOF instead."
            ),
        })
    }
}
