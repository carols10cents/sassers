use sass::output_style::{SassOutputStyle, Streamable};
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::comment::SassComment;
use ast::expression::Expression;
use token_offset::TokenOffset;
use error::{Result};

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Rule(SassRule),
    Property(TokenOffset, Expression),
    Variable(SassVariable),
    Comment(SassComment),
}

impl Streamable for Node {
    fn stream(&self, output: &mut Write, style: &SassOutputStyle)
              -> Result<()> {
        match *self {
            Node::Rule(ref sr) => try!(sr.stream(output, style)),
            Node::Variable(..) => {}, // variable declarations never get output
            Node::Property(ref name, ref expression) => {
                let ref n = name.token.to_string();
                let ref v = expression.to_string();
                try!(write!(output, "{}", style.property(n, v)));
            },
            Node::Comment(ref sc) => {
                try!(write!(output, "{}", style.before_comment()));
                try!(sc.stream(output, style));
            },
        }
        Ok(())
    }
}
