use event::{Event, Entity, State};
use std::borrow::Cow::Borrowed;

fn is_ascii_whitespace(c: u8) -> bool {
   is_newline(c) || is_ascii_whitespace_no_nl(c)
}

fn is_ascii_whitespace_no_nl(c: u8) -> bool {
    c == b'\t' || c == 0x0b || c == 0x0c || c == b' '
}

fn is_newline(c: u8) -> bool {
    c == b'\n' || c == b'\r'
}

fn isnt_newline(c: u8) -> bool {
    !is_newline(c)
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
pub struct SassRule<'a> {
    selectors: Vec<Event<'a>>,
    children: Vec<Event<'a>>,
}

impl<'a> SassRule<'a> {
    pub fn new() -> SassRule<'a> {
        SassRule {
            selectors: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    sass: &'a str,
    bytes: &'a [u8],
    offset: usize,
    stack: Vec<Entity>,
    state: State,
    sass_rule_stack: Vec<SassRule<'a>>,
    current_sass_rule: SassRule<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(sass: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            sass: &sass,
            bytes: &sass.as_bytes(),
            offset: 0,
            stack: Vec::new(),
            state: State::OutsideRules,
            sass_rule_stack: Vec::new(),
            current_sass_rule: SassRule::new(),
        }
    }

    fn start_something(&mut self) -> Option<Event<'a>> {
        match self.state {
            State::OutsideRules => {
                self.pick_something();
                println!("picked {:?}", self.state);
                return self.start_something();
            },
            State::InRule => return Some(self.next_rule()),
            State::InSelectors => return Some(self.next_selector()),
            State::InProperties => return Some(self.next_property()),
            State::InVariable => return Some(self.next_variable()),
            State::InComment => return Some(self.next_comment()),
            State::Eof => return None,
        }
    }

    fn pick_something(&mut self) {
        self.skip_leading_whitespace();

        if self.offset == self.sass.len() {
            self.state = State::Eof;
            return
        }

        let c = self.bytes[self.offset];
        let d = self.bytes[self.offset + 1];

        // if c == b'}' {
        //     self.offset += 1;
        //     return Some(self.end())
        // }

        if c == b'$' {
            self.state = State::InVariable;
            return
        }

        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return
        }

        self.state = State::InSelectors;
        self.stack.push(Entity::Rule);
    }

    fn end(&mut self) -> Event<'a> {
        let rule = match self.stack.pop() {
            Some(r) => r,
            None => {
                println!("Unexpected empty stack!");
                return Event::End(Entity::Rule)
            },
        };

        if self.stack.len() == 0 {
            self.state = State::InRule;
        } else {
            self.state = State::InProperties;
        }

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
        let mut i = self.offset;
        let limit = self.sass.len();

        while i < limit {
            let c = self.bytes[i];
            if is_ascii_whitespace(c) {
                i += self.scan_while(&self.sass[i..self.sass.len()], is_ascii_whitespace);
            } else if c == b'/' && i + 1 < limit && self.bytes[i + 1] == b'/' {
                i += self.scan_while(&self.sass[i..self.sass.len()], isnt_newline);
            } else {
                self.offset = i;
                return
            }
        }
        self.offset = limit;
    }

    fn next_comment(&mut self) -> Event<'a> {
        let comment_body_beginning = self.offset;
        let mut i = comment_body_beginning + 2;
        let limit = self.sass.len();

        while i < limit {
            match self.bytes[i..limit].iter().position(|&c| c == b'*' ) {
                Some(pos) => { i += pos; },
                None => { break; },
            }

            if self.bytes[i+1] == b'/' {
                self.offset = i + 2;
                return Event::Comment(Borrowed(&self.sass[comment_body_beginning..i + 2]))
            } else {
                i += 1;
            }
        }
        unreachable!()
    }

    fn next_variable(&mut self) -> Event<'a> {
        // TODO: can parts of this be deduplicated with properties?
        let name_beginning = self.offset;
        let mut i = name_beginning;
        let limit = self.sass.len();

        while i < limit {
            match self.bytes[i..limit].iter().position(|&c| c == b':' ) {
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
                match self.bytes[i..limit].iter().position(|&c| c == b';') {
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

        let name_beginning = self.offset;
        let mut i = name_beginning;
        let limit = self.sass.len();

        let c = self.bytes[i];
        if c == b'}' {
            self.offset += 1;
            return self.end()
        }

        let d = self.bytes[i + 1];
        if c == b'/' && d == b'*' {
            return self.next_comment()
        }

        while i < limit {
            match self.bytes[i..limit].iter().position(|&c| c == b':' || c == b'{') {
                Some(pos) => { i += pos; },
                None => { break; },
            }

            // Inefficient since we already skipped the whitespace and we'll have to
            // do it again but oh well
            let c = self.bytes[i];
            if c == b'{' {
                self.state = State::InSelectors;
                self.stack.push(Entity::Rule);
                return Event::Start(Entity::Rule)
            }

            let name_end = i;

            i += 1;
            self.offset = i;
            self.skip_leading_whitespace();

            let value_beginning = self.offset;
            i = value_beginning;

            while i < limit {
                match self.bytes[i..limit].iter().position(|&c| c == b';') {
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
        let beginning = self.offset;
        let mut i = beginning;
        let limit = self.sass.len();

        while i < limit {
            match self.bytes[i..limit].iter().position(|&c| c == b',' || c == b'{' || c == b':') {
                Some(pos) => { i += pos; },
                None => { i = limit; break; },
            }

            let c = self.bytes[i];

            println!("c = {:?}", c);

            if c == b':' {
                self.state = State::InProperties;
                return self.next_property()
            }

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

    fn next_rule(&mut self) -> Event<'a> {
        Event::Start(Entity::Rule)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = SassRule<'a>;

    fn next(&mut self) -> Option<SassRule<'a>> {
        if self.offset < self.sass.len() {
            self.start_something();
        }
        None
    }
}
