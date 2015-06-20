use std::borrow::Cow;
use std::fmt;

#[derive(Clone)]
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

impl<'a> fmt::Debug for SassRule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children = self.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().connect("\n");
        let indented_children = children.split("\n").collect::<Vec<_>>().connect("\n  ");
        write!(f, "{:?} {{\n  {}\n}}", self.selectors, indented_children)
    }
}

#[derive(Debug,Clone)]
pub struct SassVariable<'a> {
    pub variable: Event<'a>,
}

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Event<'a>,
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
    Comment(SassComment<'a>),
}
