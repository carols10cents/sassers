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
}