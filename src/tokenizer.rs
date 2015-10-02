use error::{SassError, ErrorKind, Result};
use event::Event;
use sass::comment::SassComment;
use sass::rule::SassRule;
use sass::selector::SassSelector;
use sass::variable::SassVariable;
use top_level_event::TopLevelEvent;
use tokenizer_utils::*;

use std::borrow::Cow::Borrowed;

#[derive(Debug)]
pub struct Tokenizer<'a> {
    toker: Toker<'a>,
    state: State,
    sass_rule_stack: Vec<SassRule<'a>>,
    current_sass_rule_selectors_done: bool,
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum State {
    OutsideRules,
    InVariable,
    InComment,
    InRule,
    InSelectors,
    InProperties,
    EndRule,
    Eof,
}

impl<'a> Tokenizer<'a> {
    pub fn new(inner_str: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            toker: Toker {
                inner_str: &inner_str,
                bytes: &inner_str.as_bytes(),
                offset: 0,
            },
            state: State::OutsideRules,
            sass_rule_stack: Vec::new(),
            current_sass_rule_selectors_done: false,
        }
    }

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn start_something(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let mut current_sass_rule = SassRule::new();
        self.current_sass_rule_selectors_done = false;

        self.pick_something();
        while self.state != State::OutsideRules {
            if self.state == State::Eof { return Ok(None) }

            // See if this works now that State is Copy
            // self.state = match self.state

            if self.state == State::InSelectors {
                match try!(self.next_selector()) {
                    Some(sel) => current_sass_rule.selectors.push(sel),
                    None => {},
                }
            } else if self.state == State::InProperties {
                match try!(self.next_property()) {
                    Some(prop) => current_sass_rule.children.push(prop),
                    None => {},
                }
            } else if self.state == State::EndRule {
                try!(self.toker.eat("}"));

                match self.sass_rule_stack.pop() {
                    Some(mut rule) => {
                        rule.children.push(Event::ChildRule(current_sass_rule));
                        current_sass_rule = rule;
                        self.pick_something();
                    },
                    None => self.state = State::OutsideRules,
                }
            } else if self.state == State::InRule {
                self.sass_rule_stack.push(current_sass_rule);
                current_sass_rule = SassRule::new();
                self.current_sass_rule_selectors_done = false;
                self.pick_something();
            } else if self.state == State::InVariable {
                return self.next_variable()
            } else if self.state == State::InComment {
                let comment = try!(self.next_comment());
                if comment.is_some() {
                    if self.sass_rule_stack.len() == 0 &&
                       current_sass_rule.selectors.len() == 0 &&
                       current_sass_rule.children.len() == 0 {
                           return Ok(Some(
                               TopLevelEvent::Comment(SassComment { comment: comment.unwrap() })
                           ))
                    } else {
                        current_sass_rule.children.push(comment.unwrap());
                        self.pick_something();
                    }
                } else {
                    return Ok(None)
                }
            } else {
                return Err(SassError {
                    kind: ErrorKind::TokenizerError,
                    message: format!(
                        "Something unexpected happened in tokenization! Current tokenization state: {:?}. Current sass rule = {:?}",
                        self.state,
                        current_sass_rule
                    ),
                })
            }
        }

        Ok(Some(TopLevelEvent::Rule(current_sass_rule)))
    }

    fn pick_something(&mut self) {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            self.state = State::Eof;
            return
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'}' {
            self.state = State::EndRule;
            return
        }

        if c == b'$' {
            self.state = State::InVariable;
            return
        }

        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                self.state = State::InComment;
                return
            }
        }

        if self.current_sass_rule_selectors_done {
            self.state = State::InProperties;
            return
        }

        self.state = State::InSelectors;
    }

    fn next_comment(&mut self) -> Result<Option<Event<'a>>> {
        let comment_body_beginning = self.toker.offset;
        let mut i = comment_body_beginning + 2;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, isnt_asterisk);
            self.toker.offset = i;

            if self.toker.eat("*/").is_ok() {
                return Ok(Some(
                    Event::Comment(Borrowed(
                        &self.toker.inner_str[comment_body_beginning..self.toker.offset]
                    ))
                ))
            } else {
                i += 1;
            }
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected comment; reached EOF instead."
            ),
        })
    }

    fn next_variable(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        // TODO: can parts of this be deduplicated with properties?
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            self.toker.offset = i;
            try!(self.toker.eat(":"));
            self.toker.skip_leading_whitespace();

            let value_beginning = self.toker.offset;
            i = value_beginning;

            while i < self.limit() {
                i += self.toker.scan_while_or_end(i, isnt_semicolon);
                let value_end = i;
                self.toker.offset = i;
                try!(self.toker.eat(";"));
                self.toker.skip_leading_whitespace();

                return Ok(Some(TopLevelEvent::Variable(SassVariable{
                    name: Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                    value: Borrowed(&self.toker.inner_str[value_beginning..value_end]),
                })))
            }
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected variable declaration and value; reached EOF instead."
            ),
        })
    }

    fn next_property(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            self.state = State::Eof;
            return Ok(None)
        }

        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        let c = self.toker.bytes[i];
        if c == b'}' {
            self.state = State::EndRule;
            return Ok(None)
        }

        let d = self.toker.bytes[i + 1];
        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return Ok(None)
        }

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);

            // Inefficient since we already skipped the whitespace and we'll have to
            // do it again but oh well
            let c = self.toker.bytes[i];
            if c == b'{' {
                self.state = State::InRule;
                return Ok(None)
            }

            let name_end = i;
            self.toker.offset = i;
            try!(self.toker.eat(":"));
            self.toker.skip_leading_whitespace();

            let value_beginning = self.toker.offset;
            i = value_beginning;

            while i < self.limit() {
                i += self.toker.scan_while_or_end(i, isnt_semicolon);
                let value_end = i;
                self.toker.offset = i;
                try!(self.toker.eat(";"));
                self.toker.skip_leading_whitespace();

                if self.toker.bytes[name_beginning] == b'$' {
                    return Ok(Some(Event::Variable(SassVariable {
                        name: Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                        value: Borrowed(&self.toker.inner_str[value_beginning..value_end])
                    })))
                } else {
                    return Ok(Some(Event::UnevaluatedProperty(
                        Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                        Borrowed(&self.toker.inner_str[value_beginning..value_end]),
                    )))
                }
            }
        }
        self.toker.offset = self.limit();
        Ok(None)
    }

    fn next_selector(&mut self) -> Result<Option<SassSelector<'a>>> {
        self.toker.skip_leading_whitespace();

        let beginning = self.toker.offset;
        let mut i = beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_selector_char);
            let c = self.toker.bytes[i];

            if c == b':' {
                self.state = State::InProperties;
                return Ok(None)
            }

            if c == b',' || c == b'{' {
                let n = scan_trailing_whitespace(&self.toker.inner_str[beginning..i]);
                let end = i - n;
                if end > beginning {
                    if c == b'{' {
                        if self.current_sass_rule_selectors_done {
                            self.state = State::InRule;
                            return Ok(None)
                        } else {
                            self.current_sass_rule_selectors_done = true;
                            self.state = State::InProperties;
                        }
                    }
                    self.toker.offset = i + 1;
                    return Ok(Some(SassSelector::new(&self.toker.inner_str[beginning..end])))
                } else {
                    // only whitespace between commas
                    self.toker.offset += 1;
                    return self.next_selector()
                }
            }

            self.toker.offset = i;
            if i > beginning {
                return Ok(Some(SassSelector::new(&self.toker.inner_str[beginning..i])))
            }
            i += 1;
        }

        if i > beginning {
            self.toker.offset = i;
            Ok(Some(SassSelector::new(&self.toker.inner_str[beginning..i])))
        } else {
            self.state = State::Eof;
            Ok(None)
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<TopLevelEvent<'a>>;

    fn next(&mut self) -> Option<Result<TopLevelEvent<'a>>> {
        if !self.toker.at_eof() {
            return match self.start_something() {
                Ok(Some(t)) => Some(Ok(t)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        None
    }
}
