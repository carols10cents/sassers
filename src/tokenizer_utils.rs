use error::{SassError, ErrorKind, Result};
use std::cmp;
use std::borrow::Cow::{self, Borrowed};

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

pub fn isnt_end_curly_brace(c: u8) -> bool {
    c != b'}'
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
    pub bytes: &'a [u8],
    pub offset: usize,
}

impl<'a> Toker<'a> {
    pub fn limit(&self) -> usize {
        self.inner_str.len()
    }

    pub fn at_eof(&self) -> bool {
        self.offset == self.limit()
    }

    pub fn curr_byte(&self) -> u8 {
        self.bytes[self.offset]
    }

    pub fn next_byte(&self) -> u8 {
        self.bytes[self.offset + 1]
    }

    pub fn eat(&mut self, expected: &str) -> Result<bool> {
        let original_offset = self.offset;
        for c in expected.as_bytes().iter() {
            if !try!(self.eatch(c)) {
                self.offset = original_offset;
                return Err(SassError {
                    kind: ErrorKind::TokenizerError,
                    message: format!(
                        "Expected: `{}`, Saw: `{}`",
                        expected,
                        String::from_utf8_lossy(&self.bytes[
                            self.offset..cmp::min(self.offset + expected.len(), self.limit())
                        ])
                    ),
                })
            }
        }
        Ok(true)
    }

    fn eatch(&mut self, expected_char: &u8) -> Result<bool> {
        if self.at_eof() {
            Err(SassError {
                kind: ErrorKind::UnexpectedEof,
                message: format!(
                    "Expected: `{}`; reached EOF instead.",
                    *expected_char as char
                ),
            })
        } else {
            if self.bytes[self.offset] == *expected_char {
                self.offset += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    pub fn scan_while_or_end<F>(&mut self, start: usize, f: F) -> usize
            where F: Fn(u8) -> bool {
        let end = self.limit();
        self.scan_while(&self.inner_str[start..end], f)
    }

    fn scan_while<F>(&mut self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
        }
    }

   pub fn skip_leading_whitespace(&mut self) {
       let mut i = self.offset;

       while i < self.limit() {
           let c = self.bytes[i];
           if is_ascii_whitespace(c) {
               i += self.scan_while_or_end(i, is_ascii_whitespace);
           } else if c == b'/' && i + 1 < self.limit() && self.bytes[i + 1] == b'/' {
               i += self.scan_while_or_end(i, isnt_newline);
           } else {
               self.offset = i;
               return
           }
       }
       self.offset = self.limit();
   }

   pub fn next_name(&mut self) -> Result<Cow<'a, str>> {
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
           return Ok(Borrowed(&self.inner_str[name_beginning..name_end]))
       }
       self.offset = self.limit();
       Err(SassError {
           kind: ErrorKind::UnexpectedEof,
           message: String::from(
               "Expected a valid name; reached EOF instead."
           ),
       })
   }
}
