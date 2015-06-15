use tokenizer::Tokenizer;
use event::{Event, Rule};
use variable_mapper::VariableMapper;

pub fn nested<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = Event<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output = String::from_str("");
    let mut last = Event::End(Rule::SassRule);

    while let Some(token) = vm.next() {
        let print_token = match token.clone() {
            Event::Start(_) => continue,
            Event::Selector(name) => format!("{} ", name),
            Event::Property(name, value) => {
                match last {
                    Event::Selector(_) => format!("{{\n  {}: {};", name, value),
                    _ => format!("\n  {}: {};", name, value),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => format!(" }}\n"),
                }
            },
            Event::Variable(..) => unreachable!(),
        };
        output.push_str(print_token.as_str());
        last = token;
    }
    output
}

pub fn compressed<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = Event<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output =  String::from_str("");
    let mut last = Event::End(Rule::SassRule);

    while let Some(token) = vm.next() {
        let print_token = match token.clone() {
            Event::Start(_) => continue,
            Event::Selector(name) => {
                match last {
                    Event::Selector(_) => format!(" {}", name),
                    _ => format!("{}", name),
                }
            },
            Event::Property(name, value) => {
                match last {
                    Event::Selector(_) => format!("{{{}:{}", name, value),
                    Event::Property(_, _) => format!(";{}:{}", name, value),
                    _ => format!("{}:{}", name, value),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => format!("}}"),
                }
            },
            Event::Variable(..) => unreachable!(),
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
    let mut vm = VariableMapper::new(tokenizer);
    expanded_inner(&mut vm, &mut parents)
}

fn expanded_inner<'a, I>(tokenizer: &mut I, parents: &mut Vec<String>) -> String
    where I: Iterator<Item = Event<'a>>
{
    let mut last = Event::End(Rule::SassRule);
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
            Event::Selector(name) => {
                parents.push((*name).to_string());
                match last {
                    Event::Selector(_) => current.push_str(&format!(" {}", name)[..]),
                    Event::End(_) => current.push_str(&format!("{}", name)[..]),
                    _ => current.push_str(&format!("{}", name)[..]),
                }
            },
            Event::Property(name, value) => {
                match last {
                    Event::Selector(_) => properties.push_str(&format!(" {{\n  {}: {};", name, value)[..]),
                    _ => properties.push_str(&format!("\n  {}: {};", name, value)[..]),
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
            Event::Variable(..) => unreachable!(),
        };

        last = token;
    }
    children
}

pub fn compact<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = Event<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output =  String::from_str("");
    let mut last = Event::End(Rule::SassRule);

    while let Some(token) = vm.next() {
        let print_token = match token.clone() {
            Event::Start(_) => continue,
            Event::Selector(name) => {
                match last {
                    Event::Selector(_) => format!(" {}", name),
                    Event::End(_) => format!("\n{}", name),
                    _ => format!("{}", name),
                }
            },
            Event::Property(name, value) => {
                match last {
                    Event::Selector(_) => format!(" {{ {}: {};", name, value),
                    _ => format!(" {}: {};", name, value),
                }
            },
            Event::End(_) => {
                match last {
                    Event::End(_) => continue,
                    _ => format!(" }}"),
                }
            },
            Event::Variable(..) => unreachable!(),
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
