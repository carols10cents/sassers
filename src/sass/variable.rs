use crate::ast::expression::Expression;
use crate::token_offset::TokenOffset;

#[derive(Clone, Debug, PartialEq)]
pub struct SassVariable {
    pub name: TokenOffset,
    pub value: Expression,
}

impl SassVariable {
    pub fn name_string(&self) -> String {
        self.name.token.to_string()
    }
}
