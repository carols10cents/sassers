use sass::op::Op;

use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(Cow<'a, str>),
    String(Cow<'a, str>),
    Number(f32),
    Operator(Op),
}
