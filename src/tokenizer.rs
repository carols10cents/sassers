use event::{Event, Rule, State};
use std::borrow::Cow::Borrowed;

fn is_ascii_whitespace(c: u8) -> bool {
    c == b'\n' || c == b'\r' || is_ascii_whitespace_no_nl(c)
}

fn is_ascii_whitespace_no_nl(c: u8) -> bool {
    c == b'\t' || c == 0x0b || c == 0x0c || c == b' '
}

// unusual among "scan" functions in that it scans from the _back_ of the string
// TODO: should also scan unicode whitespace?
fn scan_trailing_whitespace(data: &str) -> usize {
    match data.as_bytes().iter().rev().position(|&c| !is_ascii_whitespace_no_nl(c)) {
        Some(i) => i,
        None => data.len()
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    sass: &'a str,
    offset: usize,
    stack: Vec<Rule>,
    state: State,
}

impl<'a> Tokenizer<'a> {
    pub fn new(sass: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            sass: &sass,
            offset: 0,
            stack: Vec::new(),
            state: State::StartRule,
        }
    }

    fn start_rule(&mut self) -> Option<Event<'a>> {
        self.skip_leading_whitespace();

        if self.offset == self.sass.len() {
            return None
        }

        let c = self.sass.as_bytes()[self.offset];
        if c == b'}' {
            self.offset += 1;
            return Some(self.end())
        }
        if c == b'$' {
            return Some(self.next_variable())
        }

        self.state = State::InSelectors;
        self.stack.push(Rule::SassRule);

        Some(Event::Start(Rule::SassRule))
    }

    fn end(&mut self) -> Event<'a> {
        let rule = match self.stack.pop() {
            Some(r) => r,
            None => {
                println!("Unexpected empty stack!");
                return Event::End(Rule::SassRule)
            },
        };
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

    fn next_variable(&mut self) -> Event<'a> {
        // TODO: can parts of this be deduplicated with properties?
        let bytes = self.sass.as_bytes();
        let name_beginning = self.offset;
        let mut i = name_beginning;
        let limit = self.sass.len();

        while i < limit {
            match bytes[i..limit].iter().position(|&c| c == b':' ) {
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

                return Event::Variable(
                    Borrowed(&self.sass[name_beginning..name_end]),
                    Borrowed(&self.sass[value_beginning..value_end])
                )
            }
        }
        self.offset = self.sass.len();
        Event::Property(Borrowed(""), Borrowed(""))
    }

    fn next_property(&mut self) -> Event<'a> {
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
            match bytes[i..limit].iter().position(|&c| c == b':' || c == b'{') {
                Some(pos) => { i += pos; },
                None => { break; },
            }

            // Inefficient since we already skipped the whitespace and we'll have to
            // do it again but oh well
            let c = bytes[i];
            if c == b'{' {
                self.state = State::InSelectors;
                self.stack.push(Rule::SassRule);
                return Event::Start(Rule::SassRule)
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

    fn next_selector(&mut self) -> Event<'a> {
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

impl<'a> Iterator for Tokenizer<'a> {
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