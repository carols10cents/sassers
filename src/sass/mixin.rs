use event::Event;

use std::borrow::Cow::{self, Owned};

#[derive(Clone, Debug)]
pub struct SassMixin<'a> {
    pub name: Cow<'a, str>,
    pub parameters: Vec<SassMixinParameter<'a>>,
    pub children: Vec<Event<'a>>,
}

#[derive(Clone, Debug)]
pub struct SassMixinCall<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<Cow<'a, str>>,
}

#[derive(Clone, Debug)]
pub struct SassMixinParameter<'a> {
    pub name: Cow<'a, str>,
    pub default: Option<Cow<'a, str>>,
}

impl<'a> SassMixinParameter<'a> {
    pub fn new(param_str: Cow<'a, str>) -> SassMixinParameter<'a> {
        let mut parts = param_str.split(":");
        let name = Owned(parts.next().unwrap().into());
        let default = match parts.next() {
            Some(d) => Some(Owned(d.trim().into())),
            None => None,
        };
        SassMixinParameter { name: name, default: default }
    }
}
