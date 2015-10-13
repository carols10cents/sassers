use error::Result;
use tokenizer::Tokenizer;
use top_level_event::TopLevelEvent;
use substituter::Substituter;

pub fn expanded<'a, I>(tokenizer: &mut I) -> Result<String>
    where I: Iterator<Item = Result<TopLevelEvent<'a>>>
{
    let mut subber = Substituter::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = subber.next() {
        match event {
            Ok(TopLevelEvent::Rule(rule)) => {
                output.push_str(&rule.expanded());
                output.push_str("\n\n");
            },
            Ok(TopLevelEvent::Comment(comment)) => {
                output.push_str(&comment.expanded());
                output.push_str("\n");
            },
            Ok(TopLevelEvent::MixinCall(mixin_call)) => {
                // TODO
            },
            Ok(TopLevelEvent::Variable(..)) => {},
            Ok(TopLevelEvent::Mixin(..))    => {},
            Err(e) => return Err(e),
        }
    }

    Ok(output)
}

pub fn nested<'a, I>(tokenizer: &mut I) -> Result<String>
    where I: Iterator<Item = Result<TopLevelEvent<'a>>>
{
    let mut subber = Substituter::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = subber.next() {
        match event {
            Ok(TopLevelEvent::Rule(rule)) => {
                output.push_str(&rule.nested());
                output.push_str("\n\n");
            },
            Ok(TopLevelEvent::Comment(comment)) => {
                output.push_str(&comment.nested());
                output.push_str("\n");
            },
            Ok(TopLevelEvent::MixinCall(mixin_call)) => {
                // TODO
            },
            Ok(TopLevelEvent::Variable(..)) => {},
            Ok(TopLevelEvent::Mixin(..))    => {},
            Err(e) => return Err(e),
        }
    }

    Ok(output)
}

pub fn compact<'a, I>(tokenizer: &mut I) -> Result<String>
    where I: Iterator<Item = Result<TopLevelEvent<'a>>>
{
    let mut subber = Substituter::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = subber.next() {
        match event {
            Ok(TopLevelEvent::Rule(rule)) => {
                output.push_str(&rule.compact());
                output.push_str("\n\n");
            },
            Ok(TopLevelEvent::Comment(comment)) => {
                output.push_str(&comment.compact());
                output.push_str("\n");
            },
            Ok(TopLevelEvent::MixinCall(mixin_call)) => {
                // TODO
            },
            Ok(TopLevelEvent::Variable(..)) => {},
            Ok(TopLevelEvent::Mixin(..))    => {},
            Err(e) => return Err(e),
        }
    }

    Ok(output)
}

pub fn compressed<'a, I>(tokenizer: &mut I) -> Result<String>
    where I: Iterator<Item = Result<TopLevelEvent<'a>>>
{
    let mut subber = Substituter::new(tokenizer);
    let mut output = String::new();

    while let Some(event) = subber.next() {
        match event {
            Ok(TopLevelEvent::Rule(rule)) => {
                output.push_str(&rule.compressed());
            },
            Ok(TopLevelEvent::MixinCall(mixin_call)) => {
                // TODO
            },
            Ok(TopLevelEvent::Comment(..)) => {},
            Ok(TopLevelEvent::Variable(..)) => {},
            Ok(TopLevelEvent::Mixin(..))    => {},
            Err(e) => return Err(e),
        }
    }

    Ok(output)
}

pub fn debug(tokenizer: &mut Tokenizer) -> Result<String> {
    let mut output = String::new();

    while let Some(event) = tokenizer.next() {
        match event {
            Ok(ev) => output.push_str(&format!("{:?}\n", ev)),
            Err(e) => return Err(e),
        }
    }
    Ok(output)
}
