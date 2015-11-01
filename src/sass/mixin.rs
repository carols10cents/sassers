use event::Event;
use sass::parameters::{SassParameter, SassArgument};

use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct SassMixin<'a> {
    pub name: Cow<'a, str>,
    pub parameters: Vec<SassParameter<'a>>,
    pub children: Vec<Event<'a>>,
}

#[derive(Clone, Debug)]
pub struct SassMixinCall<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<SassArgument<'a>>,
}
