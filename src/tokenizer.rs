use event::{Event, State, SassComment, SassSelector, TopLevelEvent};
use sass_rule::SassRule;
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
pub struct Tokenizer<'a> {
    sass: &'a str,
    bytes: &'a [u8],
    offset: usize,
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
            state: State::OutsideRules,
            sass_rule_stack: Vec::new(),
            current_sass_rule: SassRule::new(),
        }
    }

    fn start_something(&mut self) -> Option<TopLevelEvent<'a>> {
        self.current_sass_rule = SassRule::new();

        self.pick_something();
        while self.state != State::OutsideRules {
            let curr_state = self.state.clone();
            match curr_state {
                State::Eof => {
                    return None
                },
                State::InSelectors => {
                    let sel = self.next_selector();
                    if sel.is_some() {
                        self.current_sass_rule.selectors.push(sel.unwrap());
                    }
                },
                State::InProperties => {
                    let prop = self.next_property();
                    if prop.is_some() {
                        self.current_sass_rule.children.push(prop.unwrap());
                    }
                },
                State::EndRule => {
                    // consume '}', should probably be checking that's what we actually have
                    self.offset += 1;

                    match self.sass_rule_stack.pop() {
                        Some(ref mut rule) => {
                            rule.children.push(Event::ChildRule(self.current_sass_rule.clone()));
                            self.current_sass_rule = (*rule).clone();
                            self.pick_something();
                        },
                        None => self.state = State::OutsideRules,
                    }
                },
                State::InRule => {
                    self.sass_rule_stack.push(self.current_sass_rule.clone());
                    self.current_sass_rule = SassRule::new();
                    self.pick_something();
                },
                State::InVariable => {
                    let var = self.next_variable();
                    if var.is_some() {
                        return var
                    } else {
                        // is this really what we should be doing here? reachable?
                        self.state = State::Eof;
                    }
                },
                State::InComment => {
                    let comment = self.next_comment();
                    if comment.is_some() {
                        if self.sass_rule_stack.len() == 0 &&
                           self.current_sass_rule.selectors.len() == 0 &&
                           self.current_sass_rule.children.len() == 0 {
                               return Some(TopLevelEvent::Comment(SassComment { comment: comment.unwrap() }))
                        } else {
                            self.current_sass_rule.children.push(comment.unwrap());
                            self.pick_something();
                        }
                    } else {
                        // is this really what we should be doing here? reachable?
                        self.state = State::Eof;
                    }
                }
                unknown_state => {
                    println!("i dont know what to do for {:?}", unknown_state);
                    println!("current sass rule = {:?}", self.current_sass_rule);
                    self.state = State::Eof;
                },
            }
        }

        Some(TopLevelEvent::Rule(self.current_sass_rule.clone()))
    }

    fn pick_something(&mut self) {
        self.skip_leading_whitespace();

        if self.offset == self.sass.len() {
            self.state = State::Eof;
            return
        }

        let c = self.bytes[self.offset];

        if c == b'}' {
            self.state = State::EndRule;
            return
        }

        if c == b'$' {
            self.state = State::InVariable;
            return
        }

        if c == b'/' && (self.offset + 1) < self.sass.len() {
            let d = self.bytes[self.offset + 1];
            if d == b'*' {
                self.state = State::InComment;
                return
            }
        }

        self.state = State::InSelectors;
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

    fn next_comment(&mut self) -> Option<Event<'a>> {
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
                return Some(Event::Comment(Borrowed(&self.sass[comment_body_beginning..i + 2])))
            } else {
                i += 1;
            }
        }
        None
    }

    fn next_variable(&mut self) -> Option<TopLevelEvent<'a>> {
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

                return Some(TopLevelEvent::SassVariable {
                    name: Borrowed(&self.sass[name_beginning..name_end]),
                    value: Borrowed(&self.sass[value_beginning..value_end]),
                })
            }
        }
        self.offset = self.sass.len();
        None
    }

    fn next_property(&mut self) -> Option<Event<'a>> {
        self.skip_leading_whitespace();

        if self.offset == self.sass.len() {
            self.state = State::Eof;
            return None
        }

        let name_beginning = self.offset;
        let mut i = name_beginning;
        let limit = self.sass.len();

        let c = self.bytes[i];
        if c == b'}' {
            self.state = State::EndRule;
            return None
        }

        let d = self.bytes[i + 1];
        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return None
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
                self.state = State::InRule;
                return None
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

                return Some(Event::Property(
                    Borrowed(&self.sass[name_beginning..name_end]),
                    Borrowed(&self.sass[value_beginning..value_end])
                ))

            }
        }
        self.offset = self.sass.len();
        None
    }

    fn next_selector(&mut self) -> Option<SassSelector<'a>> {
        let beginning = self.offset;
        let mut i = beginning;
        let limit = self.sass.len();

        while i < limit {
            match self.bytes[i..limit].iter().position(|&c| c == b',' || c == b'{' || c == b':') {
                Some(pos) => { i += pos; },
                None => { i = limit; break; },
            }

            let c = self.bytes[i];

            if c == b':' {
                self.state = State::InProperties;
                return None
            }

            if c == b',' || c == b'{' {
                let n = scan_trailing_whitespace(&self.sass[beginning..i]);
                let end = i - n;
                if end > beginning {
                    if c == b'{' {
                        if self.current_sass_rule.selectors_done {
                            self.state = State::InRule;
                            return None
                        } else {
                            self.current_sass_rule.selectors_done = true;
                            self.state = State::InProperties;
                        }
                    }
                    self.offset = i + 1;
                    return Some(SassSelector { name: Borrowed(&self.sass[beginning..end]) })
                }
            }

            self.offset = i;
            if i > beginning {
                return Some(SassSelector { name: Borrowed(&self.sass[beginning..i]) })
            }
            i += 1;
        }

        if i > beginning {
            self.offset = i;
            Some(SassSelector { name: Borrowed(&self.sass[beginning..i]) })
        } else {
            self.state = State::Eof;
            None
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TopLevelEvent<'a>;

    fn next(&mut self) -> Option<TopLevelEvent<'a>> {
        if self.offset < self.sass.len() {
            return self.start_something()
        }
        None
    }
}
