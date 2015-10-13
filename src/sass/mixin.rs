use event::Event;

use std::borrow::Cow::{self, Owned};

#[derive(Clone, Debug)]
pub struct SassMixin<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<SassMixinArgument<'a>>,
    pub children: Vec<Event<'a>>,
}

#[derive(Clone, Debug)]
pub struct SassMixinCall<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<Cow<'a, str>>,
}

#[derive(Clone, Debug)]
pub struct SassMixinArgument<'a> {
    pub name: Cow<'a, str>,
    pub default: Option<Cow<'a, str>>,
}

impl<'a> SassMixinArgument<'a> {
    pub fn new(arg_str: Cow<'a, str>) -> SassMixinArgument<'a> {
        let mut parts = arg_str.split(":");
        let name = Owned(parts.next().unwrap().into());
        let default = match parts.next() {
            Some(d) => Some(Owned(d.trim().into())),
            None => None,
        };
        SassMixinArgument { name: name, default: default }
    }
}
