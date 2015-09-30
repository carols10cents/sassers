use sass::color_value::ColorValue;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::op::Op;

use error::Result;

use std::borrow::Cow::Borrowed;

#[derive(Debug)]
pub struct ValueTokenizer<'a> {
    value_str: &'a str,
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ValueTokenizer<'a> {
    pub fn new(value_str: &'a str) -> ValueTokenizer<'a> {
        ValueTokenizer {
            value_str: &value_str,
            bytes: &value_str.as_bytes(),
            offset: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Option<ValuePart<'a>>> {
        self.skip_leading_whitespace();

        let start = self.offset;
        let mut i = self.offset;
        let limit = self.value_str.len();

        let ret = if start == limit {
            None
        } else if is_operator(self.bytes[start]) {
            self.offset = start + 1;
            Some(ValuePart::Operator(self.value_str[start..start + 1].parse().unwrap_or(Op::Plus)))
        } else if is_number(self.bytes[start]) {
            i += self.scan_while(&self.value_str[i..limit], is_number);
            self.offset = i;
            if i < limit && valid_unit_char(self.bytes[i]) {
                let unit_start = i;
                i += self.scan_while(&self.value_str[i..limit], valid_unit_char);
                self.offset = i;
                Some(ValuePart::Number(NumberValue::with_units(
                    self.value_str[start..unit_start].parse().unwrap_or(0.0),
                    Borrowed(&self.value_str[unit_start..i]),
                )))
            }
            else {
                Some(ValuePart::Number(NumberValue::from_scalar(
                    self.value_str[start..i].parse().unwrap_or(0.0)
                )))
            }
        } else if self.bytes[start] == b'$' {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i;
            Some(ValuePart::Variable(Borrowed(&self.value_str[start..i])))
        } else if self.bytes[start] == b'#' {
            i += 1;
            i += self.scan_while(&self.value_str[i..limit], valid_hex_char);
            self.offset = i;
            match ColorValue::from_hex(Borrowed(&self.value_str[start..i])) {
                Ok(v) => Some(ValuePart::Color(v)),
                Err(e) => return Err(e),
            }
        } else if self.eat("rgb(") {
            let r = self.eat_color_u8();
            self.eat(",");
            self.skip_leading_whitespace();
            let g = self.eat_color_u8();
            self.eat(",");
            self.skip_leading_whitespace();
            let b = self.eat_color_u8();
            self.eat(")");

            Some(ValuePart::Color(
                ColorValue {
                    red: r, green: g, blue: b,
                    computed: false,
                    original: Borrowed(&self.value_str[start..self.offset]),
                }
            ))
        } else {
            i += self.scan_while(&self.value_str[start..limit], isnt_space);
            self.offset = i;
            Some(ValuePart::String(Borrowed(&self.value_str[start..i])))
        };

        Ok(ret)
    }

    fn eat_color_u8(&mut self) -> u8 {
        let limit = self.value_str.len();
        let mut i = self.offset;
        let color_start = i;

        i += self.scan_while(&self.value_str[i..limit], is_number);
        self.offset = i;
        self.value_str[color_start..i].parse().unwrap_or(0)
    }

    fn eat(&mut self, expected: &str) -> bool {
        let original_offset = self.offset;
        for c in expected.as_bytes().iter() {
            if !self.eatch(c) {
                self.offset = original_offset;
                return false
            }
        }
        return true
    }

    fn eatch(&mut self, expected_char: &u8) -> bool {
        if self.bytes[self.offset] == *expected_char {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn scan_while<F>(&mut self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
        }
    }

    fn skip_leading_whitespace(&mut self) {
       let mut i = self.offset;
       let limit = self.value_str.len();

       while i < limit {
           let c = self.bytes[i];
           if is_space(c) {
               i += self.scan_while(&self.value_str[i..limit], is_space);
           } else {
               self.offset = i;
               return
           }
       }
       self.offset = limit;
   }
}

impl<'a> Iterator for ValueTokenizer<'a> {
    type Item = Result<ValuePart<'a>>;

    fn next(&mut self) -> Option<Result<ValuePart<'a>>> {
        if self.offset < self.value_str.len() {
            return match self.parse() {
                Ok(Some(v)) => Some(Ok(v)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        None
    }
}

fn is_space(c: u8) -> bool {
    c == b' '
}

fn isnt_space(c: u8) -> bool {
    !is_space(c)
}

fn valid_unit_char(c: u8) -> bool {
    c == b'%' || (!is_space(c) && !is_operator(c))
}

fn valid_hex_char(c: u8) -> bool {
    match c {
        b'0' ... b'9' | b'a' ... b'f' => true,
        _ => false,
    }
}

fn is_number(c: u8) -> bool {
    let result = match c {
        b'0' ... b'9' | b'.' => true,
        _ => false,
    };
    result
}

fn is_operator(c: u8) -> bool {
    match c {
        b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')' | b',' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::color_value::ColorValue;
    use sass::value_part::ValuePart;
    use sass::number_value::NumberValue;
    use sass::op::Op;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_returns_string_part() {
        let mut vt = ValueTokenizer::new("foo");
        assert_eq!(Some(Ok(ValuePart::String(Borrowed(&"foo")))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_space_separated_string_parts() {
        let mut vt = ValueTokenizer::new("foo bar");
        assert_eq!(Some(Ok(ValuePart::String(Borrowed(&"foo")))), vt.next());
        assert_eq!(Some(Ok(ValuePart::String(Borrowed(&"bar")))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variable() {
        let mut vt = ValueTokenizer::new("$foo");
        assert_eq!(Some(Ok(ValuePart::Variable(Borrowed(&"$foo")))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variables_and_string_parts() {
        let mut vt = ValueTokenizer::new("foo $bar baz $quux");
        assert_eq!(Some(Ok(ValuePart::String(Borrowed(&"foo")))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Variable(Borrowed(&"$bar")))), vt.next());
        assert_eq!(Some(Ok(ValuePart::String(Borrowed(&"baz")))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Variable(Borrowed(&"$quux")))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_number() {
        let mut vt = ValueTokenizer::new("3");
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(3.0)))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_two_numbers() {
        let mut vt = ValueTokenizer::new("3 8.9");
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(3.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(8.9)))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_with_units() {
        let mut vt = ValueTokenizer::new("3px");
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(3.0, Borrowed(&"px"))))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_with_units_but_not_parens() {
        let mut vt = ValueTokenizer::new("(3px)");
        assert_eq!(Some(Ok(ValuePart::Operator(Op::LeftParen))), vt.next());
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(3.0, Borrowed(&"px"))))),
            vt.next()
        );
        assert_eq!(Some(Ok(ValuePart::Operator(Op::RightParen))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_with_percents_as_units() {
        let mut vt = ValueTokenizer::new("10px 8%");
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(10.0, Borrowed(&"px"))))),
            vt.next()
        );
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(8.0, Borrowed(&"%"))))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_operator() {
        let mut vt = ValueTokenizer::new("+");
        assert_eq!(Some(Ok(ValuePart::Operator(Op::Plus))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_and_operators() {
        let mut vt = ValueTokenizer::new("6 + 75.2");
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(6.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::Plus))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(75.2)))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_and_operators_without_spaces() {
        let mut vt = ValueTokenizer::new("6+75.2");
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(6.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::Plus))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(75.2)))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_does_stuff_with_parens() {
        let mut vt = ValueTokenizer::new("2+(3 4)");
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(2.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::Plus))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::LeftParen))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(3.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(4.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::RightParen))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_does_stuff_with_slash_separators() {
        let mut vt = ValueTokenizer::new("15 / 3 / 5");
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(15.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::Slash))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(3.0)))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Operator(Op::Slash))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Number(NumberValue::from_scalar(5.0)))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_hex() {
        let mut vt = ValueTokenizer::new("#aabbcc");
        assert_eq!(
            Some(Ok(ValuePart::Color(ColorValue {
                red: 170, green: 187, blue: 204, computed: false, original: Borrowed("#aabbcc"),
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_short_hex() {
        let mut vt = ValueTokenizer::new("#cba");
        assert_eq!(
            Some(Ok(ValuePart::Color(ColorValue {
                red: 204, green: 187, blue: 170, computed: false, original: Borrowed("#cba"),
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_hex_is_separate_from_parens() {
        let mut vt = ValueTokenizer::new("#cba)");
        assert_eq!(
            Some(Ok(ValuePart::Color(ColorValue {
                red: 204, green: 187, blue: 170, computed: false, original: Borrowed("#cba"),
            }))),
            vt.next()
        );
        assert_eq!(Some(Ok(ValuePart::Operator(Op::RightParen))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_rgb() {
        let mut vt = ValueTokenizer::new("rgb(10,100,73)");
        assert_eq!(
            Some(Ok(ValuePart::Color(ColorValue {
                red: 10, green: 100, blue: 73,
                computed: false, original: Borrowed("rgb(10,100,73)"),
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_rgb_with_spaces() {
        let mut vt = ValueTokenizer::new("rgb(10, 100, 73)");
        assert_eq!(
            Some(Ok(ValuePart::Color(ColorValue {
                red: 10, green: 100, blue: 73,
                computed: false, original: Borrowed("rgb(10, 100, 73)"),
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }
}
