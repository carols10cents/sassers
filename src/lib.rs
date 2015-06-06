use std::borrow::Cow;
use std::borrow::Cow::{Borrowed};

mod token;

pub fn compile(sass: &str, style: &str) -> Result<(), &'static str> {
    let mut sp = SassParser::new(&sass);
    let parsed = try!(sp.parse());
    println!("{:?}", parsed);
    Ok(())
    // match style {
    //     "nested"     => Ok(parsed.nested(&sp)),
    //     "compressed" => Ok(parsed.compressed(&sp)),
    //     "expanded"   => Ok(parsed.expanded(&sp)),
    //     "compact"    => Ok(parsed.compact(&sp)),
    //     _            => Err("Unknown style:. Please specify one of nested, compressed, expanded, or compact."),
    // }
}

#[derive(PartialEq, Debug)]
enum State {
    StartRule,
    InSelectors,
    InProperties,
}

#[derive(Debug)]
enum Rule {
    SassRule,
}

#[derive(Debug)]
enum Event<'a> {
    Start(Rule),
    End(Rule),
    Selector(Cow<'a, str>),
    Property(Cow<'a, str>, Cow<'a, str>),
}

#[derive(Debug)]
struct SassParser<'a> {
    pub tokenizer: SassTokenizer<'a>,
    sass: &'a str,
}

impl<'a> SassParser<'a> {
    pub fn new(sass: &'a str) -> SassParser<'a> {
        let mut tokenizer = SassTokenizer::new(&sass);
        SassParser {
            tokenizer: tokenizer,
            sass: &sass,
        }
    }

    pub fn parse(&mut self) -> Result<(), &'static str> {
        while let Some(foo) = self.tokenizer.next() {
            println!("{:?}", foo);
        }

        Ok(())
    }
}

#[derive(Debug)]
struct SassTokenizer<'a> {
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

    pub fn next_property(&mut self) -> Event<'a> {
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
        println!("totes got here");
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
                // _ => println!("idk what state i'm in"),
            }
        }
        None
    }
}

pub fn char_at(s: &str, byte: usize) -> char {
    s[byte..].chars().next().unwrap()
}

pub fn is_whitespace(c: Option<char>) -> bool {
    match c.unwrap_or('\x00') { // None can be null for now... it's not whitespace
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false
    }
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
