use sass::color_value::ColorValue;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::function::SassFunctionCall;
use sass::parameters::SassArgument;
use sass::op::Op;
use token::Token;

use error::{SassError, Result};
use tokenizer_utils::*;

#[derive(Debug)]
pub struct ValueTokenizer<'a> {
    toker: Toker<'a>,
}

impl<'a> ValueTokenizer<'a> {
    pub fn new(inner_str: &'a str) -> ValueTokenizer<'a> {
        ValueTokenizer {
            toker: Toker {
                inner_str: inner_str,
                offset: 0,
            },
        }
    }

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    pub fn parse(&mut self) -> Result<Option<ValuePart>> {
        self.toker.skip_leading_whitespace();

        let start = self.toker.offset;
        let mut i = self.toker.offset;

        let ret = if start == self.limit() {
            None
        } else if is_operator(self.toker.bytes()[start]) {
            self.toker.offset = start + 1;
            Some(ValuePart::Operator(self.toker.inner_str[start..start + 1].parse().unwrap_or(Op::Plus)))
        } else if is_number(self.toker.bytes()[start]) {
            i += self.toker.scan_while_or_end(i, is_number);
            self.toker.offset = i;
            if i < self.limit() && valid_unit_char(self.toker.bytes()[i]) {
                let unit_start = i;
                i += self.toker.scan_while_or_end(i, valid_unit_char);
                self.toker.offset = i;
                Some(ValuePart::Number(NumberValue::with_units(
                    self.toker.inner_str[start..unit_start].parse().unwrap_or(0.0),
                    String::from(&self.toker.inner_str[unit_start..i]),
                )))
            }
            else {
                Some(ValuePart::Number(NumberValue::from_scalar(
                    self.toker.inner_str[start..i].parse().unwrap_or(0.0)
                )))
            }
        } else if self.toker.bytes()[start] == b'$' {
            i += self.toker.scan_while_or_end(i, isnt_space);
            self.toker.offset = i;
            Some(ValuePart::Variable(Token {
                value: String::from(&self.toker.inner_str[start..i]),
                offset: Some(self.toker.offset),
            }))
        } else if self.toker.eat("#").is_ok() {
            if self.toker.eat("{").is_ok() {
                i = self.toker.offset;
                let start_interpolation = i;
                i += self.toker.scan_while_or_end(i, (|c| c != b'}' ));
                self.toker.offset = i;
                try!(self.toker.eat("}"));
                Some(ValuePart::String(Token {
                    value: String::from(&self.toker.inner_str[start_interpolation..i]),
                    offset: Some(self.toker.offset),
                }))
            } else {
                i = self.toker.offset;
                i += self.toker.scan_while_or_end(i, valid_hex_char);
                self.toker.offset = i;
                let hex_token = Token {
                    value: String::from(&self.toker.inner_str[start..i]),
                    offset: Some(start),
                };
                match ColorValue::from_hex(hex_token) {
                    Ok(v) => Some(ValuePart::Color(v)),
                    Err(e) => return Err(e),
                }
            }
        } else {
            i += self.toker.scan_while_or_end(start, valid_string_char);
            self.toker.offset = i;

            if self.toker.eat("(").is_ok() {
                let name = String::from(&self.toker.inner_str[start..i]);
                let arguments = match self.toker.tokenize_list(",", ")", &valid_mixin_arg_char) {
                    Ok(args) => args,
                    Err(e) => {
                        debug!("HEEEEERE e.offset = {}, start = {}", e.offset, start);
                        return Err(SassError {
                            offset: e.offset + start,
                            message: format!("Parsing arguments for function {}\n{}", name, e.message),
                            ..e
                        })
                    }
                };

                if name == "url" {
                    Some(ValuePart::String(Token {
                        value: String::from(&self.toker.inner_str[start..self.toker.offset]),
                        offset: Some(start),
                    }))
                } else {
                    Some(ValuePart::Function(SassFunctionCall {
                        name: Token { value: name, offset: Some(start) },
                        arguments: arguments.into_iter().map(|a|
                            SassArgument::new(a)
                        ).collect(),
                    }))
                }
            } else {
                Some(ValuePart::String(Token {
                    value: String::from(&self.toker.inner_str[start..i]),
                    offset: Some(start),
                }))
            }
        };

        Ok(ret)
    }
}

impl<'a> Iterator for ValueTokenizer<'a> {
    type Item = Result<ValuePart>;

    fn next(&mut self) -> Option<Result<ValuePart>> {
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
    use sass::function::SassFunctionCall;
    use sass::parameters::SassArgument;
    use sass::op::Op;

    #[test]
    fn it_returns_string_part() {
        let mut vt = ValueTokenizer::new("foo");
        assert_eq!(Some(Ok(ValuePart::String("foo".into()))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_space_separated_string_parts() {
        let mut vt = ValueTokenizer::new("foo bar");
        assert_eq!(Some(Ok(ValuePart::String("foo".into()))), vt.next());
        assert_eq!(Some(Ok(ValuePart::String("bar".into()))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variable() {
        let mut vt = ValueTokenizer::new("$foo");
        assert_eq!(Some(Ok(ValuePart::Variable("$foo".into()))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variables_and_string_parts() {
        let mut vt = ValueTokenizer::new("foo $bar baz $quux");
        assert_eq!(Some(Ok(ValuePart::String("foo".into()))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Variable("$bar".into()))), vt.next());
        assert_eq!(Some(Ok(ValuePart::String("baz".into()))), vt.next());
        assert_eq!(Some(Ok(ValuePart::Variable("$quux".into()))), vt.next());
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
            Some(Ok(ValuePart::Number(NumberValue::with_units(3.0, String::from("px"))))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_with_units_but_not_parens() {
        let mut vt = ValueTokenizer::new("(3px)");
        assert_eq!(Some(Ok(ValuePart::Operator(Op::LeftParen))), vt.next());
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(3.0, String::from("px"))))),
            vt.next()
        );
        assert_eq!(Some(Ok(ValuePart::Operator(Op::RightParen))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_with_percents_as_units() {
        let mut vt = ValueTokenizer::new("10px 8%");
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(10.0, String::from("px"))))),
            vt.next()
        );
        assert_eq!(
            Some(Ok(ValuePart::Number(NumberValue::with_units(8.0, String::from("%"))))),
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
                red: 170, green: 187, blue: 204, alpha: None,
                computed: false, original: String::from("#aabbcc"),
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
                red: 204, green: 187, blue: 170, alpha: None,
                computed: false, original: String::from("#cba"),
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
                red: 204, green: 187, blue: 170, alpha: None,
                computed: false, original: String::from("#cba"),
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
            Some(Ok(ValuePart::Function(SassFunctionCall {
                name: "rgb".into(),
                arguments: vec![
                    SassArgument { name: None, value: String::from("10") },
                    SassArgument { name: None, value: String::from("100") },
                    SassArgument { name: None, value: String::from("73") },
                ],
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_rgb_with_spaces() {
        let mut vt = ValueTokenizer::new("rgb(10, 100, 73)");
        assert_eq!(
            Some(Ok(ValuePart::Function(SassFunctionCall {
                name: "rgb".into(),
                arguments: vec![
                    SassArgument { name: None, value: String::from("10") },
                    SassArgument { name: None, value: String::from("100") },
                    SassArgument { name: None, value: String::from("73") },
                ],
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_recognizes_rgb_with_named_arguments() {
        let mut vt = ValueTokenizer::new("rgb(255, $blue: 0, $green: 255)");
        assert_eq!(
            Some(Ok(ValuePart::Function(SassFunctionCall {
                name: "rgb".into(),
                arguments: vec![
                    SassArgument::new("255"),
                    SassArgument::new("$blue:0"),
                    SassArgument::new("$green: 255"),
                ],
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
                name: "some-func".into(),
                arguments: vec![
                    SassArgument { name: None, value: String::from("10") },
                    SassArgument { name: None, value: String::from("73") },
                ],
            }))),
            vt.next()
        );
        assert_eq!(None, vt.next());
    }
}
