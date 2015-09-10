use sass::op::Op;
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
}

impl<'a> ValuePart<'a> {
    pub fn into_owned(self) -> ValuePart<'static> {
        match self {
            ValuePart::Variable(name) => ValuePart::Variable(name.into_owned().into()),
            ValuePart::String(str) => ValuePart::String(str.into_owned().into()),
            ValuePart::Number(nv) => ValuePart::Number(nv.into_owned().into()),
            ValuePart::List(v) => ValuePart::List(v.into_iter().map(|p| p.into_owned().into()).collect::<Vec<_>>()),
            ValuePart::Operator(o) => ValuePart::Operator(o),
        }
    }

    pub fn computed_number(&self) -> bool {
        match *self {
            ValuePart::Number(ref nv) => nv.computed,
            _ => false,
        }
    }
}

impl<'a> fmt::Display for ValuePart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValuePart::Variable(ref name) => write!(f, "{}", name),
            ValuePart::String(ref str) => write!(f, "{}", str),
            ValuePart::Number(ref num) => write!(f, "{}", num),
            ValuePart::List(ref list) => {
                write!(f, "{}", list.iter().map( |l| l.to_string() ).collect::<Vec<_>>().join(" "))
            },
            ValuePart::Operator(Op::Slash) => write!(f, "/"),
            ValuePart::Operator(Op::Comma) => write!(f, ","),
            ValuePart::Operator(..) => unreachable!(),
        }

    }
}