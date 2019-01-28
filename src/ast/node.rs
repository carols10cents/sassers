use crate::ast::expression::Expression;
use crate::error::Result;
use crate::sass::comment::SassComment;
use crate::sass::output_style::{SassOutputStyle, Streamable};
use crate::sass::rule::SassRule;
use crate::sass::variable::SassVariable;
use crate::token_offset::TokenOffset;

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Rule(SassRule),
    Property(TokenOffset, Expression),
    Variable(SassVariable),
    Comment(SassComment),
}

impl Streamable for Node {
    fn stream(&self, output: &mut Write, style: &SassOutputStyle) -> Result<()> {
        match *self {
            Node::Rule(ref sr) => sr.stream(output, style)?,
            Node::Variable(..) => {} // variable declarations never get output
            Node::Property(ref name, ref expression) => {
                let ref n = name.token.to_string();
                let ref v = expression.to_string();
                write!(output, "{}", style.property(n, v))?;
            }
            Node::Comment(ref sc) => {
                write!(output, "{}", style.before_comment())?;
                sc.stream(output, style)?;
            }
        }
        Ok(())
    }
}
