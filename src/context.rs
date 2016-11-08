use ast::expression::Expression;
use sass::variable::SassVariable;
use token_offset::TokenOffset;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context {
    pub variables: HashMap<String, SassVariable>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            variables: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, variable: SassVariable) {
        self.variables.insert(variable.name_string(), variable);
    }

    pub fn get_variable(&self, token_offset: &TokenOffset) -> Option<Expression> {
        self.variables.get(
            &token_offset.token.to_string()
        ).and_then( |sv| Some(sv.value.clone()) )
    }
}