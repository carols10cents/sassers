use sass_rule::SassRule;

use std::borrow::Cow;

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Event<'a>,
}

impl <'a> SassComment<'a> {
    pub fn expanded(&self) -> String {
        match &self.comment {
            &Event::Comment(ref c) => (*c).to_string(),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct SassSelector<'a> {
    pub name: Cow<'a, str>,
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
pub enum Event<'a> {
    Property(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
    ChildRule(SassRule<'a>),
}

impl<'a> Event<'a> {
    pub fn expanded(&self) -> String {
        match (*self).clone() {
            Event::Property(name, value) => format!("  {}: {};", name, value),
            Event::Comment(comment) => format!("  {}", comment),
            Event::ChildRule(sass_rule) => sass_rule.expanded(),
        }
    }

    pub fn is_child_rule(&self) -> bool {
        match self {
            &Event::ChildRule(..) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TopLevelEvent<'a> {
    Rule(SassRule<'a>),
    SassVariable { name: Cow<'a, str>, value: Cow<'a, str> },
    Comment(SassComment<'a>),
}
