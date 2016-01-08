use error::{Result, SassError, ErrorKind};
use sass::value_part::ValuePart;
use evaluator::Evaluator;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SassParameter {
    pub name: String,
    pub default: Option<String>,
}

impl SassParameter {
    pub fn new(param_str: String) -> SassParameter {
        let mut parts = param_str.split(":");
        let name = String::from(parts.next().unwrap());
        let default = match parts.next() {
            Some(d) => Some(String::from(d.trim())),
            None => None,
        };
        SassParameter { name: name, default: default }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SassArgument {
    pub name: Option<String>,
    pub value: String,
}

impl SassArgument {
    pub fn new(arg_str: String) -> SassArgument {
        let mut parts = arg_str.rsplit(":");
        let value = String::from(parts.next().unwrap().trim());
        let name = match parts.next() {
            Some(d) => Some(String::from(d)),
            None => None,
        };
        SassArgument { name: name, value: value }
    }
}

pub fn collate_args_parameters(
    parameters: &Vec<SassParameter>,
    arguments: &Vec<SassArgument>,
    passed_variables: &HashMap<String, ValuePart>,
) -> Result<HashMap<String, ValuePart>> {

    let mut named_arguments = HashMap::new();

    for a in arguments.iter() {
        match a.name {
            Some(ref name) => {
                named_arguments.insert(name, a.value.clone());
            },
            None => {},
        }
    }

    let mut replacements = HashMap::new();

    for (i, p) in parameters.iter().enumerate() {
        let replacement_name = p.name.clone();
        let replacement_value =
            try!(named_arguments.get(&p.name).and_then( |v| Some(v.to_string()) )
                .or(arguments.get(i).and_then( |a| Some(a.value.clone()) ))
                .or(p.clone().default.and_then( |d| Some(d) ))
                .ok_or(SassError {
                    offset: 0,
                    kind: ErrorKind::ExpectedMixinArgument,
                    message: format!("Cannot find argument for mixin parameter named `{}` in arguments `{:?}`", p.name, arguments),
                }));

        let mut ev = Evaluator::new_from_string(replacement_value);
        let ev_res = try!(ev.evaluate(&passed_variables));

        replacements.insert(replacement_name, ev_res);
    }

    Ok(replacements)
}