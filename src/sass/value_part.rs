use error::Result;
use sass::op::Op;
use sass::color_value::ColorValue;
use sass::number_value::NumberValue;
use sass::function::SassFunctionCall;
use token::Token;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart {
    Variable(Token),
    String(Token),
    Number(NumberValue),
    Operator(Op),
    List(Vec<ValuePart>),
    Color(ColorValue),
    Function(SassFunctionCall),
}

impl ValuePart {
    pub fn concat_into_list(left: ValuePart, right: ValuePart) -> Result<ValuePart> {
        let list_parts = match (left, right) {
            (ValuePart::List(mut l), ValuePart::List(r)) => {
                l.extend(r);
                l
            },
            (l, ValuePart::List(r)) => {
                let mut ve = vec![l];
                ve.extend(r);
                ve
            },
            (ValuePart::List(mut l), r) => {
                l.push(r);
                l
            },
            (l, r) => {
                vec![l, r]
            },
        };
        Ok(ValuePart::List(list_parts))
    }

    pub fn computed_number(&self) -> bool {
        match *self {
            ValuePart::Number(ref nv) => nv.computed,
            _ => false,
        }
    }

    pub fn offset(&self) -> Option<usize> {
        match *self {
            ValuePart::Variable(ref t) |
            ValuePart::String(ref t) => t.offset,
            _ => None,
        }
    }

    pub fn compressed(&self) -> String {
        match *self {
            ValuePart::Color(ref color) => format!("{}", color.compressed()),
            ValuePart::List(ref list) => {
                let mut last_needs_space = false;
                let mut str = String::new();
                for item in list {
                    if last_needs_space &&
                       *item != ValuePart::Operator(Op::Slash) &&
                       *item != ValuePart::Operator(Op::Comma) &&
                       *item != ValuePart::Operator(Op::Minus) {
                        str.push_str(" ");
                    }
                    str.push_str(&item.compressed());
                    match item {
                        &ValuePart::Operator(Op::Slash) |
                        &ValuePart::Operator(Op::Comma) |
                        &ValuePart::Operator(Op::Minus) => last_needs_space = false,
                        _ => last_needs_space = true,
                    }
                }
                format!("{}", str)
            },
            _ => format!("{}", self),
        }
    }
}

impl fmt::Display for ValuePart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValuePart::Variable(ref name) => write!(f, "{}", name),
            ValuePart::String(ref str) => write!(f, "{}", str),
            ValuePart::Number(ref num) => write!(f, "{}", num),
            ValuePart::Color(ref color) => write!(f, "{}", color),
            ValuePart::List(ref list) => {
                let mut last_needs_space = false;
                let mut str = String::new();
                for item in list {
                    if last_needs_space &&
                       *item != ValuePart::Operator(Op::Slash) &&
                       *item != ValuePart::Operator(Op::Comma) &&
                       *item != ValuePart::Operator(Op::Minus) {
                        str.push_str(" ");
                    }
                    str.push_str(&item.to_string());
                    match item {
                        &ValuePart::Operator(Op::Slash) |
                        &ValuePart::Operator(Op::Minus) => last_needs_space = false,
                        _ => last_needs_space = true,
                    }
                }
                write!(f, "{}", str)
            },
            ValuePart::Operator(Op::Slash) => write!(f, "/"),
            ValuePart::Operator(Op::Comma) => write!(f, ","),
            ValuePart::Operator(Op::Minus) => write!(f, "-"),
            ValuePart::Operator(..) => unreachable!(),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::number_value::NumberValue;
    use sass::op::Op;

    #[test]
    fn it_puts_spaces_between_list_parts() {
        let list = ValuePart::List(vec![
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Number(NumberValue::from_scalar(52.0)),
        ]);
        assert_eq!("3 52", list.to_string());
    }

    #[test]
    fn it_does_not_put_spaces_before_commas() {
        let list = ValuePart::List(vec![
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Comma),
            ValuePart::Number(NumberValue::from_scalar(52.0)),
        ]);
        assert_eq!("3, 52", list.to_string());
    }

    #[test]
    fn it_does_not_put_spaces_around_slashes() {
        let list = ValuePart::List(vec![
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(52.0)),
        ]);
        assert_eq!("3/52", list.to_string());
    }
}
