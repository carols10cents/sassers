use event::Event;
use std::collections::HashMap;

pub struct VariableMapper<I> {
    tokenizer: I,
    variables: HashMap<String, String>,
}

impl<I> VariableMapper<I> {
    pub fn new(tokenizer: I) -> VariableMapper<I> {
        VariableMapper {
            tokenizer: tokenizer,
            variables: HashMap::new(),
        }
    }

    fn substitute_variables(&self, value: &str) -> String {
        value.split(' ').map(|value_part|
            match self.variables.get(value_part) {
                Some(v) => &v[..],
                None => value_part,
            }
        ).collect::<Vec<_>>().connect(" ")
    }
}

impl<'a, I> Iterator for VariableMapper<I>
    where I: Iterator<Item = Event<'a>>
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        match self.tokenizer.next() {
            Some(Event::Variable(name, value)) => {
                let val = self.substitute_variables(&value);
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(Event::Property(name, value)) => {
                let real_value = self.substitute_variables(&value);
                Some(Event::Property(name, real_value.into()))
            },
            other => other,
        }
    }
}
