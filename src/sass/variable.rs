use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct SassVariable<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
}
