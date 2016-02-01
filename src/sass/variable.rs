use token::Lexeme;
use ast::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct SassVariable {
    pub name: Lexeme,
    pub value: Expression,
}

impl SassVariable {
    pub fn name_string(&self) -> String {
        self.name.token.to_string()
    }
}