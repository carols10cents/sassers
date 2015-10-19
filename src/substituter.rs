use error::{Result, SassError, ErrorKind};
use evaluator::Evaluator;
use event::Event;
use top_level_event::TopLevelEvent;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::mixin::{SassMixin, SassMixinParameter, SassMixinCall, SassMixinArgument};

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

                let mixin_replacements = try!(collate_mixin_args(
                    &mixin_definition.parameters,
                    &mixin_call.arguments,
                ));

                println!("mixin_replacements = {:?}", mixin_replacements);

                let res = try!(replace_children_in_scope(
                    mixin_definition.children.clone(), mixin_replacements, local_mixins.clone()
                ));

                println!("res = {:?}", res);

                results.append(&mut mixin_definition.children.clone());
            },
            other => results.push(other),
        }
    }
    Ok(results)
}

fn collate_mixin_args<'a>(
    parameters: &Vec<SassMixinParameter<'a>>,
    arguments: &Vec<SassMixinArgument<'a>>) -> Result<HashMap<String, ValuePart<'a>>> {

    let mut named_arguments = HashMap::new();

    for a in arguments.iter() {
        match a.name {
            Some(ref name) => {
                named_arguments.insert(name.clone(), a.value.clone().into_owned());
            },
            None => {},
        }
    }

    let mut replacements = HashMap::new();

    for (i, p) in parameters.iter().enumerate() {
        match named_arguments.get(&p.name) {
            Some(ref value) => {
                replacements.insert(p.name.to_string(), ValuePart::String(Owned(value.to_string())));
            },
            None => {
                match arguments.get(i) {
                    Some(ref arg) => {
                        replacements.insert(p.name.to_string(), ValuePart::String(Owned(arg.value.clone().into_owned())));
                    },
                    None => {
                        match p.default {
                            Some(ref default) => {
                                replacements.insert(p.name.to_string(), ValuePart::String(Owned(default.clone().into_owned())));
                            },
                            None => {
                                return Err(SassError {
                                    kind: ErrorKind::ExpectedMixinArgument,
                                    message: format!("Cannot find argument for mixin parameter named `{}` in arguments `{:?}`", p.name, arguments),
                                })
                            }
                        }
                    },
                }
            },
        }
    }

    Ok(replacements)
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
