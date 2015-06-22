use sass::rule::SassRule;

use std::borrow::Cow;

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
