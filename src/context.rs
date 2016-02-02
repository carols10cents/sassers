use token::Lexeme;
use ast::expression::Expression;
use sass::variable::SassVariable;

use std::collections::HashMap;

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

    pub fn get_variable(&self, lexeme: &Lexeme) -> Option<Expression> {
        self.variables.get(&lexeme.token.to_string()).and_then(|sv| Some(sv.value.clone()))
    }
}