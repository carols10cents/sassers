use sass::output_style::SassOutputStyle;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::comment::SassComment;
use context::Context;
use error::{Result};

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Root {
    Rule(SassRule),
    Variable(SassVariable),
    Comment(SassComment),
}

impl Root {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Root::Rule(ref sr) => sr.stream(output, style),
            Root::Comment(ref sc) => {
                try!(sc.stream(output, style));
                match style {
                    SassOutputStyle::Compressed => Ok(()),
                    _ => Ok(try!(write!(output, "\n"))),
                }
            },
            Root::Variable(..) => Ok(()), // variable declarations never get output
        }
    }

    pub fn evaluate(self, context: &mut Context) -> Option<Root> {
        match self {
            Root::Rule(sr) => Some(Root::Rule(sr.evaluate(&context))),
            Root::Variable(sv) => {
                let evaluated_var = sv.value.evaluate(&context);
                context.add_variable(SassVariable {
                    name: sv.name,
                    value: evaluated_var,
                });
                None
            },
            Root::Comment(c) => Some(Root::Comment(c)),
        }
    }
}
