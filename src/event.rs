use sass::rule::SassRule;
use sass::variable::SassVariable;

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Property(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
    ChildRule(SassRule<'a>),
    Variable(SassVariable<'a>),
}

impl<'a> Event<'a> {
    pub fn expanded(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::ChildRule(ref sass_rule) => sass_rule.expanded(),
            Event::Variable(..) => unreachable!(),
        }
    }

    pub fn nested(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::ChildRule(ref sass_rule) => sass_rule.nested(),
            Event::Variable(..) => unreachable!(),
        }
    }

    pub fn compact(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("{}: {};", name, value),
            Event::Comment(ref comment) => (*comment).to_string(),
            Event::ChildRule(ref sass_rule) => sass_rule.compact(),
            Event::Variable(..) => unreachable!(),
        }
    }

    pub fn compressed(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("{}:{}", name, value),
            Event::Comment(..) => unreachable!(),
            Event::ChildRule(ref sass_rule) => sass_rule.compressed(),
            Event::Variable(..) => unreachable!(),
        }
    }

    pub fn is_child_rule(&self) -> bool {
        match self {
            &Event::ChildRule(..) => true,
            _ => false,
        }
    }

    pub fn is_comment(&self) -> bool {
        match self {
            &Event::Comment(..) => true,
            _ => false,
        }
    }
}
