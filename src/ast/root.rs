use crate::context::Context;
use crate::error::Result;
use crate::expression_evaluator::ExpressionEvaluator;
use crate::sass::comment::SassComment;
use crate::sass::output_style::{SassOutputStyle, Streamable};
use crate::sass::rule::SassRule;
use crate::sass::variable::SassVariable;

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Root {
    Rule(SassRule),
    Variable(SassVariable),
    Comment(SassComment),
}

impl Streamable for Root {
    fn stream(&self, output: &mut Write, style: &SassOutputStyle) -> Result<()> {
        match *self {
            Root::Rule(ref sr) => sr.stream(output, style)?,
            Root::Comment(ref sc) => {
                sc.stream(output, style)?;
                write!(output, "{}", style.after_comment())?;
            }
            Root::Variable(..) => {} // variable declarations never get output
        }

        Ok(())
    }
}

impl Root {
    pub fn evaluate(self, context: &mut Context) -> Option<Root> {
        match self {
            Root::Rule(sr) => Some(Root::Rule(sr.evaluate(&context))),
            Root::Variable(sv) => {
                let evaluated_var = ExpressionEvaluator::evaluate(sv.value, &context);
                context.add_variable(SassVariable {
                    name: sv.name,
                    value: evaluated_var,
                });
                None
            }
            Root::Comment(c) => Some(Root::Comment(c)),
        }
    }
}
