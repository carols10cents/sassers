use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct SassMixin<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<Cow<'a, str>>,
    pub body: Cow<'a, str>,
}

#[derive(Clone, Debug)]
pub struct SassMixinCall<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<Cow<'a, str>>,
}
