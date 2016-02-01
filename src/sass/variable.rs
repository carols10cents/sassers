use token::Lexeme;
use ast::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct SassVariable {
    pub name: Lexeme,
    pub value: Expression,
}
