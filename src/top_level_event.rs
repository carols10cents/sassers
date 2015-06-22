use sass::rule::SassRule;
use sass::comment::SassComment;

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum TopLevelEvent<'a> {
    Rule(SassRule<'a>),
    SassVariable { name: Cow<'a, str>, value: Cow<'a, str> },
    Comment(SassComment<'a>),
}
