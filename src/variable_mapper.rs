use evaluator::Evaluator;
use event::Event;
use top_level_event::TopLevelEvent;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::value_part::ValuePart;
use std::collections::HashMap;

pub struct VariableMapper<'vm, I> {
    tokenizer: I,
    variables: HashMap<String, ValuePart<'vm>>,
}

impl<'vm, I> VariableMapper<'vm, I> {
    pub fn new(tokenizer: I) -> VariableMapper<'vm, I> {
        VariableMapper {
            tokenizer: tokenizer,
            variables: HashMap::new(),
        }
    }

    fn replace_children_in_scope<'b>(&self, children: Vec<Event<'b>>, mut local_variables: HashMap<String, ValuePart<'b>>) -> Vec<Event<'b>> {
        children.into_iter().filter_map(|c|
            match c {
                Event::Variable(SassVariable { name, value }) => {
                    let val = Evaluator::new_from_string(&value).evaluate(&local_variables);
                    local_variables.insert((*name).to_string(), val);
                    None
                },
                Event::Property(name, value) => {
                    Some(Event::Property(
                        name,
                        Evaluator::new_from_string(
                            &value
                        ).evaluate(&local_variables).to_string().into()
                    ))
                },
                Event::ChildRule(rule) => {
                    Some(Event::ChildRule(SassRule {
                        children: self.replace_children_in_scope(
                            rule.children, local_variables.clone()
                        ), ..rule
                    }))
                },
                other => Some(other)
            }
        ).collect::<Vec<_>>()
    }
}

impl<'a, I> Iterator for VariableMapper<'a, I>
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    type Item = TopLevelEvent<'a>;

    fn next(&mut self) -> Option<TopLevelEvent<'a>> {
        match self.tokenizer.next() {
            Some(TopLevelEvent::Variable(SassVariable { name, value })) => {
                let val = Evaluator::new_from_string(
                    &value
                ).evaluate(&self.variables);
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(TopLevelEvent::Rule(sass_rule)) => {
                Some(TopLevelEvent::Rule(SassRule {
                    children: self.replace_children_in_scope(
                        sass_rule.children, self.variables.clone()
                    ), ..sass_rule
                }))
            },
            other => other,
        }
    }
}
