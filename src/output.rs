use tokenizer::Tokenizer;
use event::{Event};
use variable_mapper::VariableMapper;

// pub fn nested<'a, I>(tokenizer: &mut I) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut vm = VariableMapper::new(tokenizer);
//     let mut parents = Vec::new();
//     nested_inner(&mut vm, &mut parents)
// }
//
// pub fn nested_inner<'a, I>(tokenizer: &mut I, parents: &mut Vec<String>) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut last = Event::End(Entity::Rule);
//     let mut current = String::new();
//     let mut properties = String::new();
//     let mut children = String::new();
//
//     while let Some(token) = tokenizer.next() {
//        match token.clone() {
//             Event::Start(_) => {
//                 match children.len() {
//                     0 => children = nested_inner(tokenizer, parents),
//                     _ => {
//                         children.push_str("\n\n");
//                         children.push_str(&nested_inner(tokenizer, parents));
//                     },
//                 };
//             },
//             Event::Selector(name) => {
//                 parents.push((*name).to_string());
//                 current.push_str(&format!("{} ", name));
//             },
//             Event::Property(name, value) => {
//                 match last {
//                     Event::Selector(_) => properties.push_str(&format!("{{\n  {}: {};", name, value)),
//                     _ => properties.push_str(&format!("\n  {}: {};", name, value)),
//                 }
//             },
//             Event::End(_) => {
//                 match (properties.len(), children.len()) {
//                     (0, 0) => current.push_str(" }\n"),
//                     (_, 0) => {
//                         current.push_str(&properties);
//                         current.push_str(" }");
//                     },
//                     (0, _) => {
//                         current.push_str(&children);
//                     },
//                     (_, _) => {
//                         current.push_str(&properties);
//                         current.push_str(" }\n  ");
//                         current.push_str(&parents.connect(" "));
//                         current.push_str(" ");
//                         current.push_str(&children.split('\n').collect::<Vec<_>>().connect("\n  "));
//                     },
//                 }
//                 parents.pop();
//                 return current
//             },
//             Event::Comment(body) => current.push_str(&body),
//             Event::Variable(..) => unreachable!(),
//         };
//         last = token;
//     }
//     children
// }
//
// pub fn compressed<'a, I>(tokenizer: &mut I) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut vm = VariableMapper::new(tokenizer);
//     let mut parents = Vec::new();
//     compressed_inner(&mut vm, &mut parents)
// }
//
// fn compressed_inner<'a, I>(tokenizer: &mut I, parents: &mut Vec<String>) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut last = Event::End(Entity::Rule);
//     let mut current = String::new();
//     let mut properties = String::new();
//     let mut children = String::new();
//
//     while let Some(token) = tokenizer.next() {
//         match token.clone() {
//             Event::Start(_) => {
//                 match children.len() {
//                     0 => children = compressed_inner(tokenizer, parents),
//                     _ => {
//                         children.push_str(&compressed_inner(tokenizer, parents));
//                     },
//                 };
//             },
//             Event::Selector(name) => {
//                 parents.push((*name).to_string());
//                 match last {
//                     Event::Selector(_) => current.push_str(&format!(" {}", name)),
//                     _ => current.push_str(&format!("{}", name)),
//                 }
//             },
//             Event::Property(name, value) => {
//                 match last {
//                     Event::Selector(..) => properties.push_str(&format!("{{{}:{}", name, value)),
//                     Event::Property(..) => properties.push_str(&format!(";{}:{}", name, value)),
//                     _ => properties.push_str(&format!("{}:{}", name, value)),
//                 }
//             },
//             Event::End(_) => {
//                 match (properties.len(), children.len()) {
//                     (0, 0) => current.push_str("}"),
//                     (_, 0) => {
//                         current.push_str(&properties);
//                         current.push_str("}");
//                     },
//                     (0, _) => {
//                         current.push_str(" ");
//                         current.push_str(&children);
//                     },
//                     (_, _) => {
//                         current.push_str(&properties);
//                         current.push_str("}");
//                         current.push_str(&parents.connect(" "));
//                         current.push_str(" ");
//                         current.push_str(&children);
//                     },
//                 }
//                 parents.pop();
//                 return current
//             },
//             Event::Comment(body) => current.push_str(&body),
//             Event::Variable(..) => unreachable!(),
//         };
//         last = token;
//     }
//     children
// }
//
// pub fn expanded<'a, I>(tokenizer: &mut I) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut vm = VariableMapper::new(tokenizer);
//     let mut parents = Vec::new();
//     expanded_inner(&mut vm, &mut parents)
// }
//
// fn expanded_inner<'a, I>(tokenizer: &mut I, parents: &mut Vec<String>) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut last = Event::End(Entity::Rule);
//     let mut current = String::new();
//     let mut properties = String::new();
//     let mut children = String::new();
//
//     while let Some(token) = tokenizer.next() {
//         match token.clone() {
//             Event::Start(_) => {
//                 match children.len() {
//                     0 => children = expanded_inner(tokenizer, parents),
//                     _ => {
//                         children.push_str("\n\n");
//                         children.push_str(&expanded_inner(tokenizer, parents));
//                     },
//                 };
//             },
//             Event::Selector(name) => {
//                 parents.push((*name).to_string());
//                 match last {
//                     Event::Selector(_) => current.push_str(&format!(" {}", name)),
//                     Event::End(_) => current.push_str(&format!("{}", name)),
//                     _ => current.push_str(&format!("{}", name)),
//                 }
//             },
//             Event::Property(name, value) => {
//                 match properties.len() {
//                     0 => properties.push_str(&format!(" {{\n  {}: {};", name, value)),
//                     _ => properties.push_str(&format!("\n  {}: {};", name, value)),
//                 }
//             },
//             Event::End(_) => {
//                 match (properties.len(), children.len()) {
//                     (_, 0) => {
//                         current.push_str(&properties);
//                         current.push_str("\n}");
//                     },
//                     (0, _) => {
//                         current.push_str(" ");
//                         current.push_str(&children);
//                     },
//                     (_, _) => {
//                         current.push_str(&properties);
//                         current.push_str("\n}\n");
//                         current.push_str(&parents.connect(" "));
//                         current.push_str(" ");
//                         current.push_str(&children);
//                     },
//                 }
//                 parents.pop();
//                 return current
//             },
//             Event::Comment(body) => {
//                 match properties.len() {
//                     0 => current.push_str(&format!("{}\n", body)),
//                     _ => properties.push_str(&body),
//                 }
//             },
//             Event::Variable(..) => unreachable!(),
//         };
//
//         last = token;
//     }
//     current.push_str(&children);
//     current
// }
//
// pub fn compact<'a, I>(tokenizer: &mut I) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut vm = VariableMapper::new(tokenizer);
//     let mut parents = Vec::new();
//     compact_inner(&mut vm, &mut parents)
// }
//
// fn compact_inner<'a, I>(tokenizer: &mut I, parents: &mut Vec<String>) -> String
//     where I: Iterator<Item = Event<'a>>
// {
//     let mut last = Event::End(Entity::Rule);
//     let mut current = String::new();
//     let mut properties = String::new();
//     let mut children = String::new();
//
//     while let Some(token) = tokenizer.next() {
//         match token.clone() {
//             Event::Start(_) => {
//                 match children.len() {
//                     0 => children = compact_inner(tokenizer, parents),
//                     _ => {
//                         children.push_str("\n");
//                         children.push_str(&compact_inner(tokenizer, parents));
//                     },
//                 };
//             },
//             Event::Selector(name) => {
//                 parents.push((*name).to_string());
//                 match last {
//                     Event::Selector(_) => current.push_str(&format!(" {}", name)),
//                     Event::End(_) => current.push_str(&format!("{}", name)),
//                     _ => current.push_str(&format!("{}", name)),
//                 }
//             },
//             Event::Property(name, value) => {
//                 match last {
//                     Event::Selector(_) => properties.push_str(&format!(" {{ {}: {};", name, value)),
//                     _ => properties.push_str(&format!(" {}: {};", name, value)),
//                 }
//             },
//             Event::End(_) => {
//                 match (properties.len(), children.len()) {
//                     (0, 0) => current.push_str(" }"),
//                     (_, 0) => {
//                         current.push_str(&properties);
//                         current.push_str(" }\n");
//                     },
//                     (0, _) => {
//                         current.push_str(" ");
//                         current.push_str(&children);
//                     },
//                     (_, _) => {
//                         current.push_str(&properties);
//                         current.push_str(" }\n");
//                         current.push_str(&parents.connect(" "));
//                         current.push_str(" ");
//                         current.push_str(&children);
//                     },
//                 }
//                 parents.pop();
//                 return current
//             },
//             Event::Comment(body) => current.push_str(&body),
//             Event::Variable(..) => unreachable!(),
//         };
//         last = token;
//     }
//     children
// }

pub fn debug(tokenizer: &mut Tokenizer) -> String {
    let mut output = String::new();

    while let Some(rule) = tokenizer.next() {
        output.push_str(&format!("{:?}\n", rule));
    }
    output
}
