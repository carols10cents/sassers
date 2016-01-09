use token::Token;

#[derive(Clone, Debug, PartialEq)]
pub struct SassVariable {
    pub name: Token,
    pub value: Token,
}
