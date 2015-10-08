use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::value_part::ValuePart;
use sass::mixin::SassMixinCall;

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Property(Cow<'a, str>, ValuePart<'a>),
    UnevaluatedProperty(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
    ChildRule(SassRule<'a>),
    Variable(SassVariable<'a>),
    MixinCall(SassMixinCall<'a>),
}

impl<'a> Event<'a> {
    pub fn expanded(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.expanded()),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::ChildRule(ref sass_rule) => sass_rule.expanded(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn nested(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.nested()),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::ChildRule(ref sass_rule) => sass_rule.nested(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn compact(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("{}: {};", name, value.compact()),
            Event::Comment(ref comment) => (*comment).to_string(),
            Event::ChildRule(ref sass_rule) => sass_rule.compact(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn compressed(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => {
                format!("{}:{}", name, value.compressed())
            },
            Event::Comment(..) => unreachable!(),
            Event::ChildRule(ref sass_rule) => sass_rule.compressed(),
            ref other => format!("other = {:?}", other),
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
