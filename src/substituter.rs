use error::{Result, SassError, ErrorKind};
use evaluator::Evaluator;
use event::Event;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::mixin::SassMixin;
use sass::parameters::*;

use std::collections::HashMap;
use std::borrow::Cow::{self, Borrowed, Owned};

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

    fn replace_children_in_scope(
        &mut self,
        children: Vec<Event<'vm>>,
        passed_variables: Option<HashMap<String, ValuePart<'vm>>>,
        passed_mixins: Option<HashMap<String, SassMixin<'vm>>>) -> Result<Vec<Event<'vm>>> {

        let mut results = Vec::new();
        let mut local_variables = match passed_variables {
            Some(v) => v,
            None => HashMap::new(),
        };
        let local_mixins = match passed_mixins {
            Some(m) => m,
            None => self.mixins.clone(),
        };

        for c in children.into_iter() {
            match c {
                Event::Variable(SassVariable { name, value }) => {
                    let mut lvs = self.variables.clone();
                    lvs.extend(local_variables.clone());
                    let val = try!(owned_evaluated_value(value, &lvs));

                    let val = match val {
                        ValuePart::List(mut list) => {
                            match list.pop() {
                                Some(ValuePart::String(s)) => {
                                    if s == "!global" {
                                        self.variables.insert((*name).to_string(), ValuePart::List(list.clone()));
                                        ValuePart::List(list)
                                    } else {
                                        list.push(ValuePart::String(s));
                                        ValuePart::List(list)
                                    }
                                },
                                Some(other) => {
                                    list.push(other);
                                    ValuePart::List(list)
                                },
                                None => ValuePart::List(list)
                            }
                        }
                        other => other,
                    };

                    local_variables.insert((*name).to_string(), val);
                },
                Event::UnevaluatedProperty(name, value) => {
                    let mut lvs = self.variables.clone();
                    lvs.extend(local_variables.clone());

                    let mut ev = Evaluator::new_from_string(&value);
                    let ev_res = try!(ev.evaluate(&lvs)).into_owned();

                    let resulting_property = Event::Property(
                        name,
                        ev_res,
                    );
                    debug!("Resulting property: {:?}", resulting_property);
                    results.push(resulting_property);
                },
                Event::Rule(rule) => {
                    let mut lvs = self.variables.clone();
                    lvs.extend(local_variables.clone());

                    let res = try!(self.replace_children_in_scope(
                        rule.children, Some(lvs), Some(local_mixins.clone())
                    ));
                    let resulting_rule = Event::Rule(SassRule {
                        children: res, ..rule
                    });
                    debug!("Resulting rule: {:?}", resulting_rule);
                    results.push(resulting_rule);
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

                    let mut mixin_replacements = self.variables.clone();
                    mixin_replacements.extend(local_variables.clone());
                    let collated_args = try!(collate_args_parameters(
                        &mixin_definition.parameters,
                        &mixin_call.arguments,
                        &mixin_replacements,
                    ));
                    mixin_replacements.extend(collated_args);

                    let mut res = try!(self.replace_children_in_scope(
                        mixin_definition.children.clone(), Some(mixin_replacements), Some(local_mixins.clone())
                    ));
                    debug!("Resulting mixin replacements: {:?}", res);
                    results.append(&mut res);
                },
                other => {
                    debug!("Resulting other: {:?}", other);
                    results.push(other);
                },
            }
        }
        Ok(results)
    }
}

impl<'a, I> Iterator for Substituter<'a, I>
    where I: Iterator<Item = Result<Event<'a>>>
{
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Result<Event<'a>>> {
        match self.tokenizer.next() {
            Some(Ok(Event::Variable(SassVariable { name, value }))) => {
                let val = match owned_evaluated_value(value, &self.variables) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(Ok(Event::Mixin(mixin))) => {
                self.mixins.insert((mixin.name).to_string(), mixin);
                self.next()
            },
            Some(Ok(Event::Rule(sass_rule))) => {
                let replaced = match self.replace_children_in_scope(
                    sass_rule.children, None, None
                ) {
                    Ok(children) => children,
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(Event::Rule(SassRule {
                    children: replaced, ..sass_rule
                })))
            },
            Some(Ok(Event::MixinCall(mixin_call))) => {
                let replaced = match self.replace_children_in_scope(
                    vec![Event::MixinCall(mixin_call)], None, None
                ) {
                    Ok(children) => children,
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(Event::List(replaced)))
            },
            other => other,
        }
    }
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
