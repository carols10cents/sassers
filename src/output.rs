use tokenizer::Tokenizer;
use event::{Event, Rule};
use std::collections::HashMap;

fn substitute_variables<'a>(value: &'a str, substitutions: &'a HashMap<String, String>) -> String {
    value.split(' ').map(|value_part|
        match substitutions.get(value_part) {
            Some(v) => &v[..],
            None => value_part,
        }
    ).collect::<Vec<_>>().connect(" ")
}

pub fn nested(tokenizer: &mut Tokenizer) -> String {
    let mut output = String::from_str("");
    let mut last = Event::End(Rule::SassRule);
    let mut variables: HashMap<String, String> = HashMap::new();

    while let Some(token) = tokenizer.next() {
        let print_token = match token.clone() {
            Event::Start(_) => continue,
            Event::Variable(name, value) => {
                let val = substitute_variables(&value, &variables);
                variables.insert((*name).to_string(), val);
                continue
            },
            Event::Selector(name) => format!("{} ", name),
            Event::Property(name, value) => {
                let real_value = substitute_variables(&value, &variables);
                match last {
                    Event::Selector(_) => format!("{{\n  {}: {};", name, real_value),
                    _ => format!("\n  {}: {};", name, real_value),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => format!(" }}\n"),
                }
            },
        };
        output.push_str(print_token.as_str());
        last = token;
    }
    output
}

pub fn compressed(tokenizer: &mut Tokenizer) -> String {
    let mut output =  String::from_str("");
    let mut last = Event::End(Rule::SassRule);
    let mut variables = HashMap::new();

    while let Some(token) = tokenizer.next() {
        let print_token = match token.clone() {
            Event::Start(_) => continue,
            Event::Variable(name, value) => {
                let val = substitute_variables(&value, &variables);
                variables.insert((*name).to_string(), val);
                continue
            },
            Event::Selector(name) => {
                match last {
                    Event::Selector(_) => format!(" {}", name),
                    _ => format!("{}", name),
                }
            },
            Event::Property(name, value) => {
                let real_value = substitute_variables(&value, &variables);
                match last {
                    Event::Selector(_) => format!("{{{}:{}", name, real_value),
                    Event::Property(_, _) => format!(";{}:{}", name, real_value),
                    _ => format!("{}:{}", name, real_value),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => format!("}}"),
                }
            },
        };
        output.push_str(print_token.as_str());
        last = token;
    }
    output
}

pub fn expanded(tokenizer: &mut Tokenizer) -> String {
    let mut last = Event::End(Rule::SassRule);
    let mut variables = HashMap::new();
    let mut output = String::from_str("");
    let mut current = String::from_str("");

    while let Some(token) = tokenizer.next() {
        match token.clone() {
            Event::Start(_) => continue,
            Event::Variable(name, value) => {
                let val = substitute_variables(&value, &variables);
                variables.insert((*name).to_string(), val);
                continue
            },
            Event::Selector(name) => {
                match last {
                    Event::Selector(_) => current.push_str(&format!(" {}", name)[..]),
                    Event::End(_) => current.push_str(&format!("\n{}", name)[..]),
                    _ => current.push_str(&format!("{}", name)[..]),
                }
            },
            Event::Property(name, value) => {
                let real_value = substitute_variables(&value, &variables);
                match last {
                    Event::Selector(_) => current.push_str(&format!(" {{\n  {}: {};", name, real_value)[..]),
                    _ => current.push_str(&format!("\n  {}: {};", name, real_value)[..]),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => current.push_str(&format!("\n}}")[..]),
                };
                output.push_str(&current[..]);
                current = String::from_str("");
            },
        };

        last = token;
    }
    output
}

pub fn compact(tokenizer: &mut Tokenizer) -> String {
    let mut output =  String::from_str("");
    let mut last = Event::End(Rule::SassRule);
    let mut variables = HashMap::new();

    while let Some(token) = tokenizer.next() {
        let print_token = match token.clone() {
            Event::Start(_) => continue,
            Event::Variable(name, value) => {
                let val = substitute_variables(&value, &variables);
                variables.insert((*name).to_string(), val);
                continue
            },
            Event::Selector(name) => {
                match last {
                    Event::Selector(_) => format!(" {}", name),
                    Event::End(_) => format!("\n{}", name),
                    _ => format!("{}", name),
                }
            },
            Event::Property(name, value) => {
                let real_value = substitute_variables(&value, &variables);
                match last {
                    Event::Selector(_) => format!(" {{ {}: {};", name, real_value),
                    _ => format!(" {}: {};", name, real_value),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => format!(" }}"),
                }
            },
        };
        output.push_str(print_token.as_str());
        last = token;
    }
    output
}

pub fn debug(tokenizer: &mut Tokenizer) -> String {
    let mut output = String::from_str("");
    let mut current = String::from_str("");

    while let Some(token) = tokenizer.next() {
        current.push_str(&format!("{:?}\n", token)[..]);
        match token {
            Event::End(_) => {
                output.push_str(&current[..]);
                current = String::from_str("");
            },
            _ => {},
        }
    }
    output
}
