use sass::variable::SassVariable;

use std::borrow::Cow;
use std::borrow::Cow::Borrowed;
use std::collections::HashMap;

pub fn evaluate(original: &str, variables: &HashMap<String, String>) -> String {
    let vt = ValueTokenizer::new(original);
    original.split(' ').map(|original_part|
        match (*variables).get(original_part) {
            Some(v) => &v[..],
            None => original_part,
        }
    ).collect::<Vec<_>>().connect(" ")
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(SassVariable<'a>),
    String(Cow<'a, str>),
}

#[derive(Debug)]
pub struct ValueTokenizer<'a> {
    value_str: &'a str,
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ValueTokenizer<'a> {
    pub fn new(value_str: &'a str) -> ValueTokenizer<'a> {
        ValueTokenizer {
            value_str: &value_str,
            bytes: &value_str.as_bytes(),
            offset: 0,
        }
    }

    pub fn parse(&mut self) -> Option<ValuePart<'a>> {
        let start = self.offset;
        let mut i = self.offset;
        let limit = self.value_str.len();

        match self.bytes[i..limit].iter().position(|&c| c == b' ' ) {
            Some(pos) => { i += pos; },
            None      => { i = limit },
        }
        self.offset = i + 1;
        Some(ValuePart::String(Borrowed(&self.value_str[start..i])))
    }
}

impl<'a> Iterator for ValueTokenizer<'a> {
    type Item = ValuePart<'a>;

    fn next(&mut self) -> Option<ValuePart<'a>> {
        if self.offset < self.value_str.len() {
            return self.parse()
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_returns_string_part() {
        let mut vt = ValueTokenizer::new("foo");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_space_separated_string_parts() {
        let mut vt = ValueTokenizer::new("foo bar");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(Some(ValuePart::String(Borrowed(&"bar"))), vt.next());
        assert_eq!(None, vt.next());
    }
}
