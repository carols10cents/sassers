use error::Result;
use evaluator::Evaluator;
use event::Event;
use top_level_event::TopLevelEvent;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use std::collections::HashMap;
use std::borrow::Cow;

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
}

impl<'a, I> Iterator for VariableMapper<'a, I>
    where I: Iterator<Item = Result<TopLevelEvent<'a>>>
{
    type Item = Result<TopLevelEvent<'a>>;

    fn next(&mut self) -> Option<Result<TopLevelEvent<'a>>> {
        match self.tokenizer.next() {
            Some(Ok(TopLevelEvent::Variable(SassVariable { name, value }))) => {
                let val = match owned_evaluated_value(value, &self.variables) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(Ok(TopLevelEvent::Rule(sass_rule))) => {
                let replaced = match replace_children_in_scope(
                    sass_rule.children, self.variables.clone()
                ) {
                    Ok(children) => children,
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(TopLevelEvent::Rule(SassRule {
                    children: replaced, ..sass_rule
                })))
            },
            other => other,
        }
    }
}

fn replace_children_in_scope<'b>(children: Vec<Event<'b>>, mut local_variables: HashMap<String, ValuePart<'b>>) -> Result<Vec<Event<'b>>> {
    children.into_iter().filter_map(|c|
        match c {
            Event::Variable(SassVariable { name, value }) => {
                let val = match owned_evaluated_value(value, &local_variables) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                local_variables.insert((*name).to_string(), val);
                None
            },
            Event::UnevaluatedProperty(name, value) => {
                let mut ev = Evaluator::new_from_string(&value);
                let ev_res = match ev.evaluate(&local_variables) {
                    Ok(s)  => s.into_owned(),
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(Event::Property(
                    name,
                    ev_res,
                )))
            },
            Event::ChildRule(rule) => {
                let res = match replace_children_in_scope(
                    rule.children, local_variables.clone()
                ) {
                    Ok(children) => children,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok(Event::ChildRule(SassRule {
                    children: res, ..rule
                })))
            },
            other => Some(Ok(other))
        }
    ).collect()
}

fn owned_evaluated_value<'a>(
    value: Cow<'a, str>,
    variables: &HashMap<String, ValuePart<'a>>) -> Result<ValuePart<'a>> {

    let value_part = match value {
        Cow::Borrowed(v) => {
            try!(Evaluator::new_from_string(&v).evaluate(variables))
        },
        Cow::Owned(v) => {
            try!(Evaluator::new_from_string(&v).evaluate(variables)).into_owned()
        },
    };
    Ok(match value_part {
        ValuePart::Number(nv) => ValuePart::Number(NumberValue { computed: true, ..nv }),
        other => other,
    })
}
