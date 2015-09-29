use error::{Result}; //, SassError, ErrorKind};
use sass::op::Op;
use sass::color_value::ColorValue;
use sass::number_value::NumberValue;

use std::fmt;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(Cow<'a, str>),
    String(Cow<'a, str>),
    Number(NumberValue<'a>),
    Operator(Op),
    List(Vec<ValuePart<'a>>),
    Color(ColorValue<'a>),
}

impl<'a> ValuePart<'a> {
    pub fn concat_into_list(left: ValuePart<'a>, right: ValuePart<'a>) -> Result<ValuePart<'static>> {
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
        Ok(ValuePart::List(list_parts).into_owned().into())
    }

    pub fn into_owned(self) -> ValuePart<'static> {
        match self {
            ValuePart::Variable(name) => ValuePart::Variable(name.into_owned().into()),
            ValuePart::String(str) => ValuePart::String(str.into_owned().into()),
            ValuePart::Number(nv) => ValuePart::Number(nv.into_owned().into()),
            ValuePart::List(v) => ValuePart::List(v.into_iter().map(|p| p.into_owned().into()).collect::<Vec<_>>()),
            ValuePart::Operator(o) => ValuePart::Operator(o),
            ValuePart::Color(c) => ValuePart::Color(c.into_owned().into()),
        }
    }

    pub fn computed_number(&self) -> bool {
        match *self {
            ValuePart::Number(ref nv) => nv.computed,
            _ => false,
        }
    }

    pub fn expanded(&self) -> String {
        format!("{}", self)
    }

    pub fn nested(&self) -> String {
        format!("{}", self)
    }

    pub fn compact(&self) -> String {
        format!("{}", self)
    }

    pub fn compressed(&self) -> String {
        format!("{}", self)
    }
}

impl<'a> fmt::Display for ValuePart<'a> {
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
                       *item != ValuePart::Operator(Op::Comma) {
                        str.push_str(" ");
                    }
                    str.push_str(&item.to_string());
                    match item {
                        &ValuePart::Operator(Op::Slash) => last_needs_space = false,
                        _ => last_needs_space = true,
                    }
                }
                write!(f, "{}", str)
            },
            ValuePart::Operator(Op::Slash) => write!(f, "/"),
            ValuePart::Operator(Op::Comma) => write!(f, ","),
            ValuePart::Operator(..) => unreachable!(),
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
