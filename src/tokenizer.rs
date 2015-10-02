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
                let sel = self.next_selector();
                if sel.is_some() {
                    current_sass_rule.selectors.push(sel.unwrap());
                }
            } else if self.state == State::InProperties {
                let prop = self.next_property();
                if prop.is_some() {
                    current_sass_rule.children.push(prop.unwrap());
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
                return match self.next_variable() {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(e),
                }
            } else if self.state == State::InComment {
                let comment = self.next_comment();
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
                        "Current tokenization state: {:?}. Current sass rule = {:?}",
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

    fn next_comment(&mut self) -> Option<Event<'a>> {
        let comment_body_beginning = self.toker.offset;
        let mut i = comment_body_beginning + 2;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, isnt_asterisk);

            if self.toker.bytes[i+1] == b'/' {
                self.toker.offset = i + 2;
                return Some(Event::Comment(Borrowed(&self.toker.inner_str[comment_body_beginning..i + 2])))
            } else {
                i += 1;
            }
        }
        None
    }

    fn next_variable(&mut self) -> Result<TopLevelEvent<'a>> {
        // TODO: can parts of this be deduplicated with properties?
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;

            i += 1;
            self.toker.offset = i;
            self.toker.skip_leading_whitespace();

            let value_beginning = self.toker.offset;
            i = value_beginning;

            while i < self.limit() {
                i += self.toker.scan_while_or_end(i, isnt_semicolon);
                let value_end = i;
                self.toker.offset = i + 1;

                self.toker.skip_leading_whitespace();

                return Ok(TopLevelEvent::Variable(SassVariable{
                    name: Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                    value: Borrowed(&self.toker.inner_str[value_beginning..value_end]),
                }))
            }
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::ExpectedVariable,
            message: String::from(
                "Expected variable declaration and value; reached EOF instead."
            ),
        })
    }

    fn next_property(&mut self) -> Option<Event<'a>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            self.state = State::Eof;
            return None
        }

        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        let c = self.toker.bytes[i];
        if c == b'}' {
            self.state = State::EndRule;
            return None
        }

        let d = self.toker.bytes[i + 1];
        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return None
        }

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);

            // Inefficient since we already skipped the whitespace and we'll have to
            // do it again but oh well
            let c = self.toker.bytes[i];
            if c == b'{' {
                self.state = State::InRule;
                return None
            }

            let name_end = i;

            i += 1;
            self.toker.offset = i;
            self.toker.skip_leading_whitespace();

            let value_beginning = self.toker.offset;
            i = value_beginning;

            while i < self.limit() {
                i += self.toker.scan_while_or_end(i, isnt_semicolon);
                let value_end = i;
                self.toker.offset = i + 1;

                self.toker.skip_leading_whitespace();

                if self.toker.bytes[name_beginning] == b'$' {
                    return Some(Event::Variable(SassVariable {
                        name: Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                        value: Borrowed(&self.toker.inner_str[value_beginning..value_end])
                    }))
                } else {
                    return Some(Event::UnevaluatedProperty(
                        Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                        Borrowed(&self.toker.inner_str[value_beginning..value_end]),
                    ))
                }
            }
        }
        self.toker.offset = self.limit();
        None
    }

    fn next_selector(&mut self) -> Option<SassSelector<'a>> {
        self.toker.skip_leading_whitespace();

        let beginning = self.toker.offset;
        let mut i = beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_selector_char);
            let c = self.toker.bytes[i];

            if c == b':' {
                self.state = State::InProperties;
                return None
            }

            if c == b',' || c == b'{' {
                let n = scan_trailing_whitespace(&self.toker.inner_str[beginning..i]);
                let end = i - n;
                if end > beginning {
                    if c == b'{' {
                        if self.current_sass_rule_selectors_done {
                            self.state = State::InRule;
                            return None
                        } else {
                            self.current_sass_rule_selectors_done = true;
                            self.state = State::InProperties;
                        }
                    }
                    self.toker.offset = i + 1;
                    return Some(SassSelector::new(&self.toker.inner_str[beginning..end]))
                } else {
                    // only whitespace between commas
                    self.toker.offset += 1;
                    return self.next_selector()
                }
            }

            self.toker.offset = i;
            if i > beginning {
                return Some(SassSelector::new(&self.toker.inner_str[beginning..i]))
            }
            i += 1;
        }

        if i > beginning {
            self.toker.offset = i;
            Some(SassSelector::new(&self.toker.inner_str[beginning..i]))
        } else {
            self.state = State::Eof;
            None
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
