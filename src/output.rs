use tokenizer::Tokenizer;
use top_level_event::TopLevelEvent;
use variable_mapper::VariableMapper;

pub fn expanded<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = vm.next() {
        match event.clone() {
            TopLevelEvent::Rule(rule) => {
                output.push_str(&rule.expanded());
                output.push_str("\n\n");
            },
            TopLevelEvent::Variable(..) => {},
            TopLevelEvent::Comment(comment) => {
                output.push_str(&comment.expanded());
                output.push_str("\n");
            },
        }
    }

    output
}

pub fn nested<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = vm.next() {
        match event.clone() {
            TopLevelEvent::Rule(rule) => {
                output.push_str(&rule.nested());
                output.push_str("\n\n");
            },
            TopLevelEvent::Variable(..) => {},
            TopLevelEvent::Comment(comment) => {
                output.push_str(&comment.nested());
                output.push_str("\n");
            },
        }
    }

    output
}

pub fn compact<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = vm.next() {
        match event.clone() {
            TopLevelEvent::Rule(rule) => {
                output.push_str(&rule.compact());
                output.push_str("\n\n");
            },
            TopLevelEvent::Variable(..) => {},
            TopLevelEvent::Comment(comment) => {
                output.push_str(&comment.compact());
                output.push_str("\n");
            },
        }
    }

    output
}

pub fn compressed<'a, I>(tokenizer: &mut I) -> String
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    let mut vm = VariableMapper::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = vm.next() {
        match event.clone() {
            TopLevelEvent::Rule(rule) => {
                output.push_str(&rule.compressed());
            },
            TopLevelEvent::Variable(..) => {},
            TopLevelEvent::Comment(..) => {},
        }
    }

    output
}

pub fn debug(tokenizer: &mut Tokenizer) -> String {
    let mut output = String::new();

    while let Some(event) = tokenizer.next() {
        output.push_str(&format!("{:?}\n", event));
    }
    output
}
