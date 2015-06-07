#![feature(collections)]
#![feature(convert)]

use std::borrow::Cow;
use std::borrow::Cow::{Borrowed};

pub fn compile(sass: &str, style: &str) -> Result<String, &'static str> {
    let mut st = SassTokenizer::new(&sass);

    // while let Some(foo) = st.next() {
    //     println!("{:?}", foo);
    // }

    match style {
        "nested"     => Ok(nested_output(&mut st)),
        "compressed" => Ok(compressed_output(&mut st)),
    //     "expanded"   => Ok(parsed.expanded(&sp)),
    //     "compact"    => Ok(parsed.compact(&sp)),
        _            => Err("Unknown style:. Please specify one of nested, compressed, expanded, or compact."),
    }
}

pub fn nested_output(tokenizer: &mut SassTokenizer) -> String {
    let mut output =  String::from_str("");
    while let Some(token) = tokenizer.next() {
        let print_token = match token {
            Event::Start(_) => continue,
            Event::Selector(name) => format!("{} {{", name),
            Event::Property(name, value) => format!("\n  {}: {};", name, value),
            Event::End(_) => format!(" }}\n"),
        };
        output.push_str(print_token.as_str());
    }
    output
}

pub fn compressed_output(tokenizer: &mut SassTokenizer) -> String {
    let mut output =  String::from_str("");
    while let Some(token) = tokenizer.next() {
        let print_token = match token {
            Event::Start(_) => continue,
            Event::Selector(name) => format!("{}{{", name),
            Event::Property(name, value) => format!("{}:{}", name, value),
            Event::End(_) => format!("}}"),
        };
        output.push_str(print_token.as_str());
    }
    output
}

#[derive(PartialEq, Debug)]
pub enum State {
    StartRule,
    InSelectors,
    InProperties,
}

#[derive(Debug)]
pub enum Rule {
    SassRule,
}

#[derive(Debug)]
pub enum Event<'a> {
    Start(Rule),
    End(Rule),
    Selector(Cow<'a, str>),
    Property(Cow<'a, str>, Cow<'a, str>),
}

#[derive(Debug)]
pub struct SassTokenizer<'a> {
    sass: &'a str,
    offset: usize,
    stack: Vec<Rule>,
    state: State,
}

impl<'a> SassTokenizer<'a> {
    pub fn new(sass: &'a str) -> SassTokenizer<'a> {
        SassTokenizer {
            sass: &sass,
            offset: 0,
            stack: Vec::new(),
            state: State::StartRule,
        }
    }

    pub fn start_rule(&mut self) -> Option<Event<'a>> {
        self.state = State::InSelectors;
        self.stack.push(Rule::SassRule);

        Some(Event::Start(Rule::SassRule))
    }

    fn end(&mut self) -> Event<'a> {
        let rule = self.stack.pop().unwrap();
        self.state = State::StartRule;
        Event::End(rule)
    }

    fn scan_while<F>(&mut self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
        }
    }

    fn skip_leading_whitespace(&mut self) {
        let i = self.offset;
        self.offset += self.scan_while(&self.sass[i..self.sass.len()], is_ascii_whitespace);
    }

    pub fn next_property(&mut self) -> Event<'a> {
        self.skip_leading_whitespace();

        let bytes = self.sass.as_bytes();
        let name_beginning = self.offset;
        let mut i = name_beginning;
        let limit = self.sass.len();

        let c = bytes[i];
        if c == b'}' {
            self.offset += 1;
            return self.end()
        }

        while i < limit {
            match bytes[i..limit].iter().position(|&c| c == b':') {
                Some(pos) => { i += pos; },
                None => { break; },
            }

            let name_end = i;

            i += 1;
            self.offset = i;
            self.skip_leading_whitespace();

            let value_beginning = self.offset;
            i = value_beginning;

            while i < limit {
                match bytes[i..limit].iter().position(|&c| c == b';') {
                    Some(pos) => { i += pos; },
                    None => { i = limit; break; },
                }

                let value_end = i;
                self.offset = i + 1;

                self.skip_leading_whitespace();

                return Event::Property(
                    Borrowed(&self.sass[name_beginning..name_end]),
                    Borrowed(&self.sass[value_beginning..value_end])
                )

            }
        }
        self.offset = self.sass.len();
        Event::Property(Borrowed(""), Borrowed(""))
    }

    pub fn next_selector(&mut self) -> Event<'a> {
        let bytes = self.sass.as_bytes();
        let beginning = self.offset;
        let mut i = beginning;
        let limit = self.sass.len();

        while i < limit {
            match bytes[i..limit].iter().position(|&c| c == b',' || c == b'{') {
                Some(pos) => { i += pos; },
                None => { i = limit; break; },
            }

            let c = bytes[i];

            if c == b',' || c == b'{' {
                let n = scan_trailing_whitespace(&self.sass[beginning..i]);
                let end = i - n;
                if end > beginning {
                    if c == b'{' {
                        self.state = State::InProperties;
                    }
                    self.offset = i + 1;
                    return Event::Selector(Borrowed(&self.sass[beginning..end]));
                }
            }

            self.offset = i;
            if i > beginning {
                return Event::Selector(Borrowed(&self.sass[beginning..i]))
            }
            i += 1;
        }

        if i > beginning {
            self.offset = i;
            Event::Selector(Borrowed(&self.sass[beginning..i]))
        } else {
            self.end()
        }
    }
}

impl<'a> Iterator for SassTokenizer<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if self.offset < self.sass.len() {
            match self.state {
                State::StartRule => {
                    let ret = self.start_rule();
                    if ret.is_some() {
                        return ret
                    }
                },
                State::InSelectors => return Some(self.next_selector()),
                State::InProperties => return Some(self.next_property()),
            }
        }
        None
    }
}

pub fn char_at(s: &str, byte: usize) -> char {
    s[byte..].chars().next().unwrap()
}

pub fn is_ascii_whitespace(c: u8) -> bool {
    c == b'\n' || c == b'\r' || is_ascii_whitespace_no_nl(c)
}

pub fn is_ascii_whitespace_no_nl(c: u8) -> bool {
    c == b'\t' || c == 0x0b || c == 0x0c || c == b' '
}

// unusual among "scan" functions in that it scans from the _back_ of the string
// TODO: should also scan unicode whitespace?
pub fn scan_trailing_whitespace(data: &str) -> usize {
    match data.as_bytes().iter().rev().position(|&c| !is_ascii_whitespace_no_nl(c)) {
        Some(i) => i,
        None => data.len()
    }
}
