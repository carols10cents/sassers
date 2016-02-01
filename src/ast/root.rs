use sass::output_style::SassOutputStyle;
use sass::rule::SassRule;
use token::Lexeme;
use ast::expression::Expression;
use error::{Result};

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Root {
    Rule(SassRule),
    Variable(Lexeme, Expression),
}

impl Root {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Root::Rule(ref sr) => sr.stream(output, style),
            Root::Variable(..) => Ok(()), // variable declarations never get output
        }
    }
}
