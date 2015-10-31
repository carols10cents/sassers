use std::borrow::Cow::{self, Owned};

#[derive(Clone, Debug, PartialEq)]
pub struct SassFunctionCall<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<SassFunctionArgument<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SassFunctionArgument<'a> {
    pub name: Option<Cow<'a, str>>,
    pub value: Cow<'a, str>,
}

impl<'a> SassFunctionArgument<'a> {
    pub fn new(arg_str: Cow<'a, str>) -> SassFunctionArgument<'a> {
        let mut parts = arg_str.rsplit(":");
        let value = Owned(parts.next().unwrap().trim().into());
        let name = match parts.next() {
            Some(d) => Some(Owned(d.into())),
            None => None,
        };
        SassFunctionArgument { name: name, value: value }
    }
}
