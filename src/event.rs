use std::borrow::Cow;

#[derive(PartialEq, Debug)]
pub enum State {
    OutsideRules,
    InVariable,
    InComment,
    InRule,
    InSelectors,
    InProperties,
    Eof,
}

#[derive(Debug, Clone)]
pub enum Entity {
    Rule,
    Selectors
}

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Start(Entity),
    End(Entity),
    Selector(Cow<'a, str>),
    Property(Cow<'a, str>, Cow<'a, str>),
    Variable(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
}
