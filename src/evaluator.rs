use ast::expression::Expression;
use ast::root::Root;
use token::{Token, Lexeme};
use context::Context;

use std::collections::HashMap;

pub fn evaluate(root: Root, context: &mut Context) -> Option<Root> {
    match root {
        Root::Rule(sr) => Some(Root::Rule(sr.evaluate(&context))),
        Root::Variable(sv) => {
            context.add_variable(sv);
            None
        },
    }
}
