use std::borrow::Cow;

#[derive(PartialEq, Debug)]
pub enum State {
    StartRule,
    InSelectors,
    InProperties,
}

#[derive(Debug, Clone)]
pub enum Rule {
    SassRule,
}

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Start(Rule),
    End(Rule),
    Selector(Cow<'a, str>),
    Property(Cow<'a, str>, Cow<'a, str>),
    Variable(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
}
