use error::{Result, SassError, ErrorKind};
use sass::value_part::ValuePart;
use evaluator::Evaluator;

use std::borrow::Cow::{self, Owned};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SassParameter<'a> {
    pub name: Cow<'a, str>,
    pub default: Option<Cow<'a, str>>,
}

impl<'a> SassParameter<'a> {
    pub fn new(param_str: Cow<'a, str>) -> SassParameter<'a> {
        let mut parts = param_str.split(":");
        let name = Owned(parts.next().unwrap().into());
        let default = match parts.next() {
            Some(d) => Some(Owned(d.trim().into())),
            None => None,
        };
        SassParameter { name: name, default: default }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SassArgument<'a> {
    pub name: Option<Cow<'a, str>>,
    pub value: Cow<'a, str>,
}

impl<'a> SassArgument<'a> {
    pub fn new(arg_str: Cow<'a, str>) -> SassArgument<'a> {
        let mut parts = arg_str.rsplit(":");
        let value = Owned(parts.next().unwrap().trim().into());
        let name = match parts.next() {
            Some(d) => Some(Owned(d.into())),
            None => None,
        };
        SassArgument { name: name, value: value }
    }
}

pub fn collate_args_parameters<'a>(
    parameters: &Vec<SassParameter<'a>>,
    arguments: &Vec<SassArgument<'a>>,
    passed_variables: &HashMap<String, ValuePart<'a>>,
) -> Result<HashMap<String, ValuePart<'a>>> {

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
        let replacement_name = p.name.to_string();

        let replacement_value =
            try!(named_arguments.get(&p.name).and_then( |v| Some(v.to_string()) )
                .or(arguments.get(i).and_then( |a| Some(a.value.clone().into_owned()) ))
                .or(p.default.clone().and_then( |d| Some(d.into_owned()) ))
                .ok_or(SassError {
                    kind: ErrorKind::ExpectedMixinArgument,
                    message: format!("Cannot find argument for mixin parameter named `{}` in arguments `{:?}`", p.name, arguments),
                }));

        let mut ev = Evaluator::new_from_string(&replacement_value);
        let ev_res = try!(ev.evaluate(&passed_variables)).into_owned();

        replacements.insert(replacement_name, ev_res);
    }

    Ok(replacements)
}