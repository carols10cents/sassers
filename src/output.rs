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

pub fn expanded<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = Event<'a>>
{
    let mut parents = Vec::new();
    expanded_inner(tokenizer, &mut parents)
}

fn expanded_inner<'a, I>(tokenizer: &mut I, parents: &mut Vec<String>) -> String
    where I: Iterator<Item = Event<'a>>
{
    let mut last = Event::End(Rule::SassRule);
    let mut variables = HashMap::new();
    let mut current = String::from_str("");
    let mut properties = String::from_str("");
    let mut children = String::from_str("");

    while let Some(token) = tokenizer.next() {
        match token.clone() {
            Event::Start(_) => {
                match children.len() {
                    0 => children = expanded_inner(tokenizer, parents),
                    _ => {
                        children.push_str("\n\n");
                        children.push_str(&expanded_inner(tokenizer, parents)[..]);
                    },
                };
            },
            Event::Variable(name, value) => {
                let val = substitute_variables(&value, &variables);
                variables.insert((*name).to_string(), val);
                continue
            },
            Event::Selector(name) => {
                parents.push((*name).to_string());
                match last {
                    Event::Selector(_) => current.push_str(&format!(" {}", name)[..]),
                    Event::End(_) => current.push_str(&format!("{}", name)[..]),
                    _ => current.push_str(&format!("{}", name)[..]),
                }
            },
            Event::Property(name, value) => {
                let real_value = substitute_variables(&value, &variables);
                match last {
                    Event::Selector(_) => properties.push_str(&format!(" {{\n  {}: {};", name, real_value)[..]),
                    _ => properties.push_str(&format!("\n  {}: {};", name, real_value)[..]),
                }
            },
            Event::End(_) => {
                match (properties.len(), children.len()) {
                    (0, 0) => current.push_str("\n}"),
                    (_, 0) => {
                        current.push_str(&properties[..]);
                        current.push_str("\n}");
                    },
                    (0, _) => {
                        current.push_str(" ");
                        current.push_str(&children[..]);
                    },
                    (_, _) => {
                        current.push_str(&properties[..]);
                        current.push_str("\n}\n");
                        current.push_str(&parents.connect(" "));
                        current.push_str(" ");
                        current.push_str(&children[..]);
                    },
                }
                parents.pop();
                return current
            },
        };

        last = token;
    }
    children
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
