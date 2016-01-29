use sass::output_style::SassOutputStyle;
use token::Lexeme;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    List(Vec<Lexeme>),
}

impl Expression {
    pub fn to_string(&self, style: SassOutputStyle) -> String {
        match *self {
            Expression::List(ref elements) => elements.iter().map(|e|
                e.token.to_string()
            ).collect::<Vec<_>>().join(" "),
        }
    }
}