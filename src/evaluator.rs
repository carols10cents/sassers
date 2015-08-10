use std::str::FromStr;
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
            ValuePart::Number(n) => n.to_string(),
            ValuePart::Operator(..) => unreachable!(), // Not doing anything with operators atm
        }
    ).collect::<Vec<_>>().connect(" ")
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(Cow<'a, str>),
    String(Cow<'a, str>),
    Number(f32),
    Operator(Op),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LeftParen,
    RightParen,
    Comma,
}

impl FromStr for Op {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Op::Plus),
            "-" => Ok(Op::Minus),
            "*" => Ok(Op::Star),
            "/" => Ok(Op::Slash),
            "%" => Ok(Op::Percent),
            "(" => Ok(Op::LeftParen),
            ")" => Ok(Op::RightParen),
            "," => Ok(Op::Comma),
            _   => Err(()),
        }
    }
}

fn isnt_space(c: u8) -> bool {
    c != b' '
}

fn is_number(c: u8) -> bool {
    let result = match c {
        b'0' ... b'9' | b'.' => true,
        _ => false,
    };
    result
}

fn is_operator(c: u8) -> bool {
    match c {
        b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')' | b',' => true,
        _ => false,
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

        if is_operator(self.bytes[start]) {
            self.offset = start + 2;
            Some(ValuePart::Operator(self.value_str[start..start + 1].parse().unwrap()))
        } else if is_number(self.bytes[start]) {
            i += self.scan_while(&self.value_str[i..limit], is_number);
            self.offset = i + 1;
            Some(ValuePart::Number(self.value_str[start..i].parse().unwrap()))
        } else if self.bytes[start] == b'$' {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i + 1;
            Some(ValuePart::Variable(Borrowed(&self.value_str[start..i])))
        } else {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i + 1;
            Some(ValuePart::String(Borrowed(&self.value_str[start..i])))
        }
    }

    fn scan_while<F>(&mut self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
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
    fn it_returns_number() {
        let mut vt = ValueTokenizer::new("3");
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_two_numbers() {
        let mut vt = ValueTokenizer::new("3 8.9");
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(Some(ValuePart::Number(8.9)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_operator() {
        let mut vt = ValueTokenizer::new("+");
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_and_operators() {
        let mut vt = ValueTokenizer::new("6 + 75.2");
        assert_eq!(Some(ValuePart::Number(6.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(Some(ValuePart::Number(75.2)), vt.next());
        assert_eq!(None, vt.next());
    }

    // evaluate tests =====================

    #[test]
    fn it_subtitutes_variable_values() {
        let mut vars = HashMap::new();
        vars.insert("$bar".to_string(), "4".to_string());
        vars.insert("$quux".to_string(), "3px 10px".to_string());

        let answer = evaluate("foo $bar 199.82 baz $quux", &vars);
        assert_eq!("foo 4 199.82 baz 3px 10px", answer);
    }
}
