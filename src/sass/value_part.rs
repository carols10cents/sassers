use sass::op::Op;

use std::fmt;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(Cow<'a, str>),
    String(Cow<'a, str>),
    Number(f32),
    NumberUnits(f32, Cow<'a, str>),
    Operator(Op),
    List(Vec<ValuePart<'a>>),
    Computed(f32),
}

impl<'a> fmt::Display for ValuePart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValuePart::Variable(ref name) => write!(f, "{}", name),
            ValuePart::String(ref str) => write!(f, "{}", str),
            ValuePart::Number(ref num) => write!(f, "{}", num),
            ValuePart::NumberUnits(ref num, ref units) => write!(f, "{}{}", num, units),
            ValuePart::List(ref list) => {
                write!(f, "{}", list.iter().map( |l| l.to_string() ).collect::<Vec<_>>().join(" "))
            },
            ValuePart::Computed(ref vp) => write!(f, "{}", vp),
            ValuePart::Operator(Op::Slash) => write!(f, "/"),
            ValuePart::Operator(..) => unreachable!(),
        }

    }
}