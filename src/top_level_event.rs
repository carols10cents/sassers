use sass::rule::SassRule;
use sass::comment::SassComment;
use sass::variable::SassVariable;

#[derive(Debug, Clone)]
pub enum TopLevelEvent<'a> {
    Rule(SassRule<'a>),
    Variable(SassVariable<'a>),
    Comment(SassComment<'a>),
}
