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
}