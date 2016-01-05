use event::Event;
use sass::parameters::{SassParameter, SassArgument};

#[derive(Clone, Debug)]
pub struct SassMixin {
    pub name: String,
    pub parameters: Vec<SassParameter>,
    pub children: Vec<Event>,
}

#[derive(Clone, Debug)]
pub struct SassMixinCall {
    pub name: String,
    pub arguments: Vec<SassArgument>,
}
