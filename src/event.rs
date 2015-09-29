use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::value_part::ValuePart;

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Property(Cow<'a, str>, ValuePart<'a>),
    UnevaluatedProperty(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
    ChildRule(SassRule<'a>),
    Variable(SassVariable<'a>),
}

impl<'a> Event<'a> {
    pub fn expanded(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.expanded()),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::ChildRule(ref sass_rule) => sass_rule.expanded(),
            _ => unreachable!(),
        }
    }

    pub fn nested(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.nested()),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::ChildRule(ref sass_rule) => sass_rule.nested(),
            _ => unreachable!(),
        }
    }

    pub fn compact(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("{}: {};", name, value.compact()),
            Event::Comment(ref comment) => (*comment).to_string(),
            Event::ChildRule(ref sass_rule) => sass_rule.compact(),
            _ => unreachable!(),
        }
    }

    pub fn compressed(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => {
                format!("{}:{}", name, value.compressed().replace(", ", ","))
            },
            Event::Comment(..) => unreachable!(),
            Event::ChildRule(ref sass_rule) => sass_rule.compressed(),
            _ => unreachable!(),
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
