use sass::output_style::SassOutputStyle;
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

impl Node {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Node::Rule(ref sr) => try!(sr.stream(output, style)),
            Node::Variable(..) => {}, // variable declarations never get output
            Node::Property(ref name, ref expression) => {
                let ref n = name.token;
                let ref v = expression.to_string(style);
                // grumble mumble format strings you know they're a string literal
                let property = match style {
                    SassOutputStyle::Nested     => format!("  {}: {};", n, v),
                    SassOutputStyle::Expanded   => format!("  {}: {};", n, v),
                    SassOutputStyle::Compact    => format!("{}: {};", n, v),
                    SassOutputStyle::Compressed => format!("{}:{}", n, v),
                    SassOutputStyle::Debug      => format!("{:#?}\n", self),
                    _ => unreachable!(),
                };
                try!(write!(output, "{}", property));
            },
            Node::Comment(ref sc) => {
                match style {
                    SassOutputStyle::Nested | SassOutputStyle::Expanded => {
                        try!(write!(output, "  "));
                    },
                    SassOutputStyle::Compressed |
                    SassOutputStyle::Compact |
                    SassOutputStyle::Debug => {},
                    _ => unreachable!(),
                };
                try!(sc.stream(output, style))
            },
        }
        Ok(())
    }
}
