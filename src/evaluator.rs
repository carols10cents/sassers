use sass::variable::SassVariable;

use std::fmt;
use std::borrow::Cow;
use std::borrow::Cow::Borrowed;
use std::collections::HashMap;

pub fn evaluate(original: &str, variables: &HashMap<String, String>) -> String {
    let vt = ValueTokenizer::new(original);
    vt.into_iter().map(|part|
        match part {
            ValuePart::Variable(ref name) => {
                match (*variables).get(&(*name).to_string()) {
                    Some(v) => v.to_owned(),
                    None => name.to_string(),
                }
            },
            ValuePart::String(ref s) => s.to_string(),
        }
    ).collect::<Vec<_>>().connect(" ")
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(Cow<'a, str>),
    String(Cow<'a, str>),
}

impl<'a> fmt::Display for ValuePart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match self {
            &ValuePart::Variable(ref s) => s,
            &ValuePart::String(ref s) => s,
        };
        write!(f, "{}", val)
    }
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
        if self.bytes[start] == b'$' {
            Some(ValuePart::Variable(Borrowed(&self.value_str[start..i])))
        } else {
            Some(ValuePart::String(Borrowed(&self.value_str[start..i])))
        }
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
    use std::collections::HashMap;

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

    #[test]
    fn it_returns_variable() {
        let mut vt = ValueTokenizer::new("$foo");
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$foo"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variables_and_string_parts() {
        let mut vt = ValueTokenizer::new("foo $bar baz $quux");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$bar"))), vt.next());
        assert_eq!(Some(ValuePart::String(Borrowed(&"baz"))), vt.next());
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$quux"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_subtitutes_variable_values() {
        let mut vars = HashMap::new();
        vars.insert("$bar".to_string(), "4".to_string());
        vars.insert("$quux".to_string(), "3px 10px".to_string());
        let answer = evaluate("foo $bar baz $quux", &vars);
        assert_eq!("foo 4 baz 3px 10px", answer);
    }
}
