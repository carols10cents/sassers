use error::{Result, SassError, ErrorKind};
use sass::value_part::ValuePart;
use evaluator::Evaluator;
use token::Token;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SassParameter {
    pub name: Token,
    pub default: Option<String>,
}

impl SassParameter {
    pub fn new<T: Into<Token>>(param: T) -> SassParameter {
        let param_token = param.into();
        let param_str = param_token.value;
        let mut parts = param_str.split(":");
        let name = Token {
            value: String::from(parts.next().unwrap()),
            offset: param_token.offset,
        };
        let default = match parts.next() {
            Some(d) => Some(String::from(d.trim())),
            None => None,
        };
        SassParameter { name: name, default: default }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SassArgument {
    pub name: Option<Token>,
    pub value: String,
}

impl SassArgument {
    pub fn new<T: Into<Token>>(arg: T) -> SassArgument {
        let arg_token = arg.into();
        let arg_str = arg_token.value;
        let mut parts = arg_str.rsplit(":");
        let value = String::from(parts.next().unwrap().trim());
        let name = match parts.next() {
            Some(d) => Some(Token { value: String::from(d), offset: arg_token.offset }),
            None => None,
        };
        SassArgument { name: name, value: value }
    }
}

pub fn collate_args_parameters<'a>(
    parameters: &'a Vec<SassParameter>,
    arguments: &'a Vec<SassArgument>,
    passed_variables: &'a HashMap<Token, ValuePart>,
) -> Result<HashMap<Token, ValuePart>> {

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
                .or(p.clone().default.and_then( |d| Some(d.clone()) ))
                .ok_or(SassError {
                    offset: 0,
                    kind: ErrorKind::ExpectedMixinArgument,
                    message: format!("Cannot find argument for mixin parameter named `{}` in arguments `{:?}`", p.name, arguments),
                }));

        let value_string = replacement_value;

        debug!("Evaluator::new_from_string");

        let mut ev = Evaluator::new_from_string(&value_string);
        let ev_res = try!(ev.evaluate(&passed_variables));

        replacements.insert(replacement_name, ev_res.clone());
    }

    Ok(replacements)
}