use sass::color_value::ColorValue;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::function::{SassFunctionCall, SassFunctionArgument};
use sass::op::Op;

use error::Result;
use tokenizer_utils::*;

use std::borrow::Cow::Borrowed;

#[derive(Debug)]
pub struct ValueTokenizer<'a> {
    toker: Toker<'a>,
}

impl<'a> ValueTokenizer<'a> {
    pub fn new(inner_str: &'a str) -> ValueTokenizer<'a> {
        ValueTokenizer {
            toker: Toker {
                inner_str: &inner_str,
                bytes: &inner_str.as_bytes(),
                offset: 0,
            },
        }
    }

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    pub fn parse(&mut self) -> Result<Option<ValuePart<'a>>> {
        self.toker.skip_leading_whitespace();

        let start = self.toker.offset;
        let mut i = self.toker.offset;

        let ret = if start == self.limit() {
            None
        } else if is_operator(self.toker.bytes[start]) {
            self.toker.offset = start + 1;
            Some(ValuePart::Operator(self.toker.inner_str[start..start + 1].parse().unwrap_or(Op::Plus)))
        } else if is_number(self.toker.bytes[start]) {
            i += self.toker.scan_while_or_end(i, is_number);
            self.toker.offset = i;
            if i < self.limit() && valid_unit_char(self.toker.bytes[i]) {
                let unit_start = i;
                i += self.toker.scan_while_or_end(i, valid_unit_char);
                self.toker.offset = i;
                Some(ValuePart::Number(NumberValue::with_units(
                    self.toker.inner_str[start..unit_start].parse().unwrap_or(0.0),
                    Borrowed(&self.toker.inner_str[unit_start..i]),
                )))
            }
            else {
                Some(ValuePart::Number(NumberValue::from_scalar(
                    self.toker.inner_str[start..i].parse().unwrap_or(0.0)
                )))
            }
        } else if self.toker.bytes[start] == b'$' {
            i += self.toker.scan_while_or_end(i, isnt_space);
            self.toker.offset = i;
            Some(ValuePart::Variable(Borrowed(&self.toker.inner_str[start..i])))
        } else if self.toker.eat("#").is_ok() {
            if self.toker.eat("{").is_ok() {
                i = self.toker.offset;
                let start_interpolation = i;
                i += self.toker.scan_while_or_end(i, (|c| c != b'}' ));
                self.toker.offset = i;
                try!(self.toker.eat("}"));
                Some(ValuePart::String(Borrowed(
                    &self.toker.inner_str[start_interpolation..i]
                )))
            } else {
                i = self.toker.offset;
                i += self.toker.scan_while_or_end(i, valid_hex_char);
                self.toker.offset = i;
                match ColorValue::from_hex(Borrowed(&self.toker.inner_str[start..i])) {
                    Ok(v) => Some(ValuePart::Color(v)),
                    Err(e) => return Err(e),
                }
            }
        } else if self.toker.eat("rgb(").is_ok() {
            let r = self.eat_color_u8();
            try!(self.toker.eat(","));
            self.toker.skip_leading_whitespace();
            let g = self.eat_color_u8();
            try!(self.toker.eat(","));
            self.toker.skip_leading_whitespace();
            let b = self.eat_color_u8();
            try!(self.toker.eat(")"));

            Some(ValuePart::Color(
                ColorValue {
                    red: r, green: g, blue: b,
                    computed: false,
                    original: Borrowed(&self.toker.inner_str[start..self.toker.offset]),
                }
            ))
        } else {
            i += self.toker.scan_while_or_end(start, valid_string_char);
            self.toker.offset = i;

            if self.toker.eat("(").is_ok() {
                let name = Borrowed(&self.toker.inner_str[start..i]);
                let arguments = try!(self.toker.tokenize_list(",", ")", &valid_mixin_arg_char));

                if name == "url" {
                    Some(ValuePart::String(
                        Borrowed(&self.toker.inner_str[start..self.toker.offset])
                    ))
                } else {
                    Some(ValuePart::Function(SassFunctionCall {
                        name: name,
                        arguments: arguments.into_iter().map(|a|
                            SassFunctionArgument::new(a)
                        ).collect(),
                    }))
                }
            } else {
                Some(ValuePart::String(Borrowed(&self.toker.inner_str[start..i])))
            }
        };

        Ok(ret)
    }

    fn eat_color_u8(&mut self) -> u8 {
        let mut i = self.toker.offset;
        let color_start = i;

        i += self.toker.scan_while_or_end(i, is_number);
        self.toker.offset = i;
        self.toker.inner_str[color_start..i].parse().unwrap_or(0)
    }
}

impl<'a> Iterator for ValueTokenizer<'a> {
    type Item = Result<ValuePart<'a>>;

    fn next(&mut self) -> Option<Result<ValuePart<'a>>> {
        if !self.toker.at_eof() {
            return match self.parse() {
                Ok(Some(v)) => Some(Ok(v)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::color_value::ColorValue;
    use sass::value_part::ValuePart;
    use sass::number_value::NumberValue;
    use sass::function::{SassFunctionCall, SassFunctionArgument};
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

    #[test]
    fn it_recognizes_arbitrary_functions() {
        let mut vt = ValueTokenizer::new("some-func(10, 73)");
        assert_eq!(
            Some(Ok(ValuePart::Function(SassFunctionCall {
                name: Borrowed("some-func"),
                arguments: vec![
                    SassFunctionArgument { name: None, value: Borrowed("10") },
                    SassFunctionArgument { name: None, value: Borrowed("73") },
                ],
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }
}
