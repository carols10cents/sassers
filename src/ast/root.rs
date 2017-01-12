use sass::output_style::{SassOutputStyle, Streamable};
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::comment::SassComment;
use context::Context;
use expression_evaluator::ExpressionEvaluator;
use error::{Result};

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Root {
    Rule(SassRule),
    Variable(SassVariable),
    Comment(SassComment),
}

impl Streamable for Root {
    fn stream(&self, output: &mut Write, style: &SassOutputStyle)
              -> Result<()> {
        match *self {
            Root::Rule(ref sr) => try!(sr.stream(output, style)),
            Root::Comment(ref sc) => {
                try!(sc.stream(output, style));
                try!(write!(output, "{}", style.after_comment()));
            },
            Root::Variable(..) => {}, // variable declarations never get output
        }

        Ok(())
    }
}

impl Root {
    pub fn evaluate(self, context: &mut Context) -> Option<Root> {
        match self {
            Root::Rule(sr) => Some(Root::Rule(sr.evaluate(&context))),
            Root::Variable(sv) => {
                let evaluated_var = ExpressionEvaluator::evaluate(
                    sv.value,
                    &context
                );
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
