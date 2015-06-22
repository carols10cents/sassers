use top_level_event::TopLevelEvent;
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
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    type Item = TopLevelEvent<'a>;

    fn next(&mut self) -> Option<TopLevelEvent<'a>> {
        match self.tokenizer.next() {
            Some(TopLevelEvent::SassVariable { name, value }) => {
                let val = self.substitute_variables(&value);
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(TopLevelEvent::Rule(sass_rule)) => {
                let replacement = sass_rule.map_over_property_values(&|property_value| {
                    self.substitute_variables(&property_value).into()
                });

                Some(TopLevelEvent::Rule(replacement))
            },
            other => other,
        }
    }
}
