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

    fn replace_children_in_scope<'b>(&self, children: Vec<Event<'b>>, mut local_variables: HashMap<String, ValuePart<'b>>) -> Vec<Event<'b>> {
        children.into_iter().filter_map(|c|
            match c {
                Event::Variable(SassVariable { name, value }) => {
                    let val = owned_evaluated_value(value, &local_variables);
                    local_variables.insert((*name).to_string(), val);
                    None
                },
                Event::Property(name, value) => {
                    println!("Yup totes in here");
                    let mut ev = Evaluator::new_from_string(&value);
                    println!("ev is fine");
                    let ev_res = ev.evaluate(&local_variables);
                    println!("ev res is fine {:?}", ev_res);
                    let ev_res_string = ev_res.to_string();
                    println!("ev res string is fine");


                    let res = Some(Event::Property(
                        name,
                        // TODO: Is it ok that property values are strings or should they
                        // be value parts ...?
                        ev_res_string.into()
                    ));
                    println!("Totes make it herez");
                    res
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

fn owned_evaluated_value<'a>(
    value: Cow<'a, str>,
    variables: &HashMap<String, ValuePart<'a>>) -> ValuePart<'a> {

    let value_part = match value {
        Cow::Borrowed(v) => {
            Evaluator::new_from_string(&v).evaluate(variables)
        },
        Cow::Owned(v) => {
            Evaluator::new_from_string(&v).evaluate(variables).into_owned()
        },
    };
    match value_part {
        ValuePart::Number(nv) => ValuePart::Number(NumberValue { computed: true, ..nv }),
        other => other,
    }
}

impl<'a, I> Iterator for VariableMapper<'a, I>
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    type Item = TopLevelEvent<'a>;

    fn next(&mut self) -> Option<TopLevelEvent<'a>> {
        match self.tokenizer.next() {
            Some(TopLevelEvent::Variable(SassVariable { name, value })) => {
                let val = owned_evaluated_value(value, &self.variables);
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
