use sass::rule::SassRule;
use sass::comment::SassComment;
use sass::variable::SassVariable;
use sass::mixin::{SassMixin, SassMixinCall};

#[derive(Debug, Clone)]
pub enum TopLevelEvent<'a> {
    Rule(SassRule<'a>),
    Variable(SassVariable<'a>),
    Comment(SassComment<'a>),
    Mixin(SassMixin<'a>),
    MixinCall(SassMixinCall<'a>),
}
