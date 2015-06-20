use std::borrow::Cow;

#[derive(Debug,Clone)]
pub struct SassRule<'a> {
    pub selectors: Vec<Event<'a>>,
    pub children: Vec<Event<'a>>,
}

impl<'a> SassRule<'a> {
    pub fn new() -> SassRule<'a> {
        SassRule {
            selectors: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct SassVariable<'a> {
    pub variable: Event<'a>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    OutsideRules,
    InVariable,
    InComment,
    InRule,
    InSelectors,
    InProperties,
    EndRule,
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
    ChildRule(SassRule<'a>),
}

#[derive(Debug)]
pub enum TopLevelEvent<'a> {
    Rule(SassRule<'a>),
    Variable(SassVariable<'a>),
}
