use evaluator;
use event::Event;
use top_level_event::TopLevelEvent;
use sass::rule::SassRule;
use sass::variable::SassVariable;
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

    fn replace_children_in_scope<'b>(&self, children: Vec<Event<'b>>, starting_variables: &HashMap<String, String>) -> Vec<Event<'b>> {
        let mut local_variables = starting_variables.clone();

        children.into_iter().filter_map(|c|
            match c {
                Event::Variable(SassVariable { name, value }) => {
                    let val = evaluator::evaluate(&value, &local_variables);
                    local_variables.insert((*name).to_string(), val);
                    None
                },
                Event::Property(name, value) => {
                    Some(Event::Property(name, evaluator::evaluate(&value, &local_variables).into()))
                },
                Event::ChildRule(rule) => {
                    Some(Event::ChildRule(SassRule {
                        children: self.replace_children_in_scope(rule.children, &local_variables), ..rule
                    }))
                },
                other => Some(other)
            }
        ).collect::<Vec<_>>()
    }
}

impl<'a, I> Iterator for VariableMapper<I>
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    type Item = TopLevelEvent<'a>;

    fn next(&mut self) -> Option<TopLevelEvent<'a>> {
        match self.tokenizer.next() {
            Some(TopLevelEvent::Variable(SassVariable { name, value })) => {
                let val = evaluator::evaluate(&value, &self.variables);
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(TopLevelEvent::Rule(sass_rule)) => {
                Some(TopLevelEvent::Rule(SassRule {
                    children: self.replace_children_in_scope(sass_rule.children, &self.variables), ..sass_rule
                }))
            },
            other => other,
        }
    }
}
