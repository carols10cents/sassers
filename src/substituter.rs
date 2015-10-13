use error::{Result, SassError, ErrorKind};
use evaluator::Evaluator;
use event::Event;
use top_level_event::TopLevelEvent;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::mixin::{SassMixin, SassMixinCall};

use std::collections::HashMap;
use std::borrow::Cow;

pub struct Substituter<'vm, I> {
    tokenizer: I,
    variables: HashMap<String, ValuePart<'vm>>,
    mixins:    HashMap<String, SassMixin<'vm>>,
}

impl<'vm, I> Substituter<'vm, I> {
    pub fn new(tokenizer: I) -> Substituter<'vm, I> {
        Substituter {
            tokenizer: tokenizer,
            variables: HashMap::new(),
            mixins:    HashMap::new(),
        }
    }
}

impl<'a, I> Iterator for Substituter<'a, I>
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
            Some(Ok(TopLevelEvent::Mixin(mixin))) => {
                self.mixins.insert((mixin.name).to_string(), mixin);
                self.next()
            },
            Some(Ok(TopLevelEvent::Rule(sass_rule))) => {
                let replaced = match replace_children_in_scope(
                    sass_rule.children, self.variables.clone(), self.mixins.clone()
                ) {
                    Ok(children) => children,
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(TopLevelEvent::Rule(SassRule {
                    children: replaced, ..sass_rule
                })))
            },
            Some(Ok(TopLevelEvent::MixinCall(mixin_call))) => unimplemented!(),
            other => other,
        }
    }
}

fn replace_children_in_scope<'b>(
    children: Vec<Event<'b>>,
    mut local_variables: HashMap<String, ValuePart<'b>>,
    local_mixins: HashMap<String, SassMixin<'b>>) -> Result<Vec<Event<'b>>> {

    let mut results = Vec::new();

    for c in children.into_iter() {
        match c {
            Event::Variable(SassVariable { name, value }) => {
                let val = try!(owned_evaluated_value(value, &local_variables));
                local_variables.insert((*name).to_string(), val);
            },
            Event::UnevaluatedProperty(name, value) => {
                let mut ev = Evaluator::new_from_string(&value);
                let ev_res = try!(ev.evaluate(&local_variables)).into_owned();

                results.push(Event::Property(
                    name,
                    ev_res,
                ));
            },
            Event::ChildRule(rule) => {
                let res = try!(replace_children_in_scope(
                    rule.children, local_variables.clone(), local_mixins.clone()
                ));
                results.push(Event::ChildRule(SassRule {
                    children: res, ..rule
                }));
            },
            Event::MixinCall(mixin_call) => {
                let mixin_name = mixin_call.name.into_owned();
                let mixin_definition = match local_mixins.get(&mixin_name) {
                    Some(mixin) => mixin,
                    None => return Err(SassError {
                        kind: ErrorKind::ExpectedMixin,
                        message: format!("Cannot find mixin named `{}`", mixin_name),
                    }),
                };

                results.append(&mut mixin_definition.children.clone());
            },
            other => results.push(other),
        }
    }
    Ok(results)
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
