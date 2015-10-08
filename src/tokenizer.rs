use error::{SassError, ErrorKind, Result};
use event::Event;
use sass::comment::SassComment;
use sass::rule::SassRule;
use sass::selector::SassSelector;
use sass::variable::SassVariable;
use sass::mixin::{SassMixin, SassMixinCall, SassMixinArgument};
use top_level_event::TopLevelEvent;
use tokenizer_utils::*;

use std::borrow::Cow::Borrowed;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Tokenizer<'a> {
    toker: Toker<'a>,
    state: State,
    sass_rule_stack: Vec<SassRule<'a>>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum State {
    OutsideRules,
    InVariable,
    InComment,
    InSelectors,
    InProperties,
    InMixin,
    InMixinCall,
    EndRule,
    Eof,
}

#[derive(Debug)]
pub struct InnerTokenizer<'a> {
    toker: Toker<'a>,
    state: State,
}

impl<'a> InnerTokenizer<'a> {

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn start_something(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'}' {
            debug!("end rule set line 57");
            return Ok(None)
        }
        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                let comment = self.next_comment();
                debug!("inner next comment: {:?}", comment);
                return comment
            }
        }

        match self.state {
            State::Eof | State::EndRule => Ok(None),
            State::InProperties => self.next_property(),
            State::InSelectors => self.next_rule(),
            other => unreachable!("got {:?}", other),
        }
    }

    fn next_rule(&mut self) -> Result<Option<Event<'a>>> {
        let mut current_sass_rule = SassRule::new();
        current_sass_rule.selectors = try!(self.selector_list());
        debug!("INNER Recursing...");
        debug!("offset before = {:?}", self.toker.offset);
        let mut inner = InnerTokenizer {
            toker: Toker {
                inner_str: &self.toker.inner_str,
                bytes: &self.toker.bytes,
                offset: self.toker.offset,
            },
            state: State::InProperties,
        };
        while let Some(Ok(e)) = inner.next() {
            current_sass_rule.children.push(e);
        }
        self.toker.offset = inner.toker.offset;
        debug!("offset after = {:?}", self.toker.offset);
        self.state = State::InProperties;

        while let Some(Ok(e)) = self.next() {
            debug!("pushing {:?}", e);
            current_sass_rule.children.push(e);
        }
        try!(self.toker.eat("}"));
        debug!("INNER recursing done, children: {:?}", current_sass_rule);
        debug!("current state = {:?}", self.state);

        Ok(Some(Event::ChildRule(current_sass_rule)))
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

    fn next_property(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.curr_byte();
        if c == b'}' {
            debug!("end rule set line 145");
            return Ok(None)
        }

        let d = self.toker.next_byte();
        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return Ok(None)
        }

        let saved_offset = self.toker.offset;

        if self.toker.eat("@include ").is_ok() {
            return self.next_mixin_call()
        }

        if self.toker.eat("@extend ").is_ok() {
            return self.next_mixin_call()
        }

        let prop_name = try!(self.next_name());

        debug!("prop_name = {:?}", prop_name);

        let c = self.toker.curr_byte();
        debug!("c = {:?} ? {:?}", c as char, c == b'{');
        if c == b'{' {
            self.state = State::InSelectors;
            self.toker.offset = saved_offset;
            return match self.next() {
                Some(Ok(e))  => Ok(Some(e)),
                Some(Err(e)) => Err(e),
                None         => Ok(None),
            }
        }

        try!(self.toker.eat(":"));
        self.toker.skip_leading_whitespace();

        let prop_value = try!(self.next_value());

        try!(self.toker.eat(";"));
        self.toker.skip_leading_whitespace();

        if prop_name.as_bytes()[0] == b'$' {
            Ok(Some(Event::Variable(SassVariable {
                name:  prop_name,
                value: prop_value,
            })))
        } else {
            Ok(Some(Event::UnevaluatedProperty(
                prop_name,
                prop_value,
            )))
        }
    }

    fn next_mixin_call(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        i += self.toker.scan_while_or_end(i, valid_name_char);
        let name_end = i;
        let name = Borrowed(&self.toker.inner_str[name_beginning..name_end]);

        self.toker.offset = i;

        let arguments = if self.toker.eat("(").is_ok() {
            try!(self.tokenize_list(",", ")", &valid_mixin_arg_char))
        } else {
            Vec::new()
        };

        try!(self.toker.eat(";"));

        let mixin_call = Event::MixinCall(SassMixinCall {
            name: name,
            arguments: arguments,
        });

        return Ok(Some(mixin_call))
    }

    fn next_name(&mut self) -> Result<Cow<'a, str>> {
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        // Colons are valid at the beginning of a name
        if self.toker.eat(":").is_ok() {
            i = self.toker.offset;
        }

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            self.toker.offset = i;
            return Ok(Borrowed(&self.toker.inner_str[name_beginning..name_end]))
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected a valid name; reached EOF instead."
            ),
        })
    }

    fn next_value(&mut self) -> Result<Cow<'a, str>> {
        let value_beginning = self.toker.offset;
        let mut i = value_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, isnt_semicolon);
            let value_end = i;
            self.toker.offset = i;
            return Ok(Borrowed(&self.toker.inner_str[value_beginning..value_end]))
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected a valid value; reached EOF instead."
            ),
        })
    }

    fn tokenize_list<F>(&mut self, separator: &str, end_list: &str, valid_char_fn: &F) -> Result<Vec<Cow<'a, str>>>
        where F: Fn(u8) -> bool {
        let mut list = Vec::new();

        let mut i = self.toker.offset;
        while i < self.limit() {
            self.toker.skip_leading_whitespace();
            i = self.toker.offset;
            let beginning = self.toker.offset;
            i += self.toker.scan_while_or_end(i, valid_char_fn);

            let n = scan_trailing_whitespace(&self.toker.inner_str[beginning..i]);
            let end = i - n;

            if end > beginning {
                list.push(Borrowed(&self.toker.inner_str[beginning..end]));
            }

            self.toker.offset = i;
            if self.toker.eat(end_list).is_ok() {
                break;
            } else {
                try!(self.toker.eat(separator));
            }
        }

        Ok(list)
    }

    fn selector_list(&mut self) -> Result<Vec<SassSelector<'a>>> {
        let selectors = try!(self.tokenize_list(",", "{", &valid_selector_char));
        debug!("selector list selectors = {:?}", selectors);
        self.state = State::InProperties;

        Ok(selectors.into_iter().map(|s| SassSelector::new(s)).collect())
    }
}

impl<'a> Iterator for InnerTokenizer<'a> {
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Result<Event<'a>>> {
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
        }
    }

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn start_something(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        self.pick_something();

        while self.state != State::OutsideRules {
            if self.state == State::Eof { return Ok(None) }

            // See if this works now that State is Copy
            // self.state = match self.state

            if self.state == State::InSelectors {
                let mut current_sass_rule = SassRule::new();
                current_sass_rule.selectors = try!(self.selector_list());
                debug!("Recursing...");
                debug!("offset before = {:?}", self.toker.offset);
                let mut inner = InnerTokenizer {
                    toker: Toker {
                        inner_str: &self.toker.inner_str,
                        bytes: &self.toker.bytes,
                        offset: self.toker.offset,
                    },
                    state: State::InProperties,
                };
                while let Some(Ok(e)) = inner.next() {
                    current_sass_rule.children.push(e);
                }
                self.toker.offset = inner.toker.offset;

                try!(self.toker.eat("}"));

                match self.sass_rule_stack.pop() {
                    Some(mut rule) => {
                        rule.children.push(Event::ChildRule(current_sass_rule));
                        self.pick_something();
                    },
                    None => {
                        self.state = State::OutsideRules;
                        return Ok(Some(TopLevelEvent::Rule(current_sass_rule)))
                    },
                }
            } else if self.state == State::InVariable {
                return self.next_variable()
            } else if self.state == State::InMixin {
                return self.next_mixin()
            } else if self.state == State::InMixinCall {
                return self.next_mixin_call()
            } else if self.state == State::InComment {
                let comment = try!(self.next_comment());
                if comment.is_some() {
                   return Ok(Some(
                       TopLevelEvent::Comment(SassComment { comment: comment.unwrap() })
                   ))
                } else {
                    return Ok(None)
                }
            } else {
                return Err(SassError {
                    kind: ErrorKind::TokenizerError,
                    message: format!(
                        "Something unexpected happened in tokenization! Current tokenization state: {:?}.",
                        self.state
                    ),
                })
            }
        }

        Ok(None)
    }

    fn pick_something(&mut self) {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            self.state = State::Eof;
            return
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'}' {
            debug!("end rule set line 416");
            self.state = State::EndRule;
            return
        }

        if c == b'$' {
            self.state = State::InVariable;
            return
        }

        if self.toker.eat("@mixin ").is_ok() {
            self.state = State::InMixin;
            return
        }

        if self.toker.eat("@include ").is_ok() {
            self.state = State::InMixinCall;
            return
        }

        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                self.state = State::InComment;
                return
            }
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

    fn next_name(&mut self) -> Result<Cow<'a, str>> {
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        // Colons are valid at the beginning of a name
        if self.toker.eat(":").is_ok() {
            i = self.toker.offset;
        }

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            self.toker.offset = i;
            return Ok(Borrowed(&self.toker.inner_str[name_beginning..name_end]))
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected a valid name; reached EOF instead."
            ),
        })
    }

    fn next_value(&mut self) -> Result<Cow<'a, str>> {
        let value_beginning = self.toker.offset;
        let mut i = value_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, isnt_semicolon);
            let value_end = i;
            self.toker.offset = i;
            return Ok(Borrowed(&self.toker.inner_str[value_beginning..value_end]))
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected a valid value; reached EOF instead."
            ),
        })
    }

    fn next_variable(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let var_name = try!(self.next_name());

        try!(self.toker.eat(":"));
        self.toker.skip_leading_whitespace();

        let var_value = try!(self.next_value());

        try!(self.toker.eat(";"));
        self.toker.skip_leading_whitespace();

        Ok(Some(TopLevelEvent::Variable(SassVariable {
            name:  var_name,
            value: var_value,
        })))
    }

    fn next_mixin(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;

            self.toker.offset = i;
            try!(self.toker.eat("("));

            let arguments = try!(self.tokenize_list(",", ")", &valid_mixin_arg_char));

            self.toker.skip_leading_whitespace();
            try!(self.toker.eat("{"));
            self.toker.skip_leading_whitespace();
            i = self.toker.offset;
            let body_beginning = i;
            i += self.toker.scan_while_or_end(i, isnt_end_curly_brace);
            let body_end = i;
            self.toker.offset = i;
            try!(self.toker.eat("}"));

            let mixin = TopLevelEvent::Mixin(SassMixin {
                name: Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                arguments: arguments.into_iter().map(|a|
                    SassMixinArgument::new(a)
                ).collect(),
                body: Borrowed(&self.toker.inner_str[body_beginning..body_end]),
            });

            return Ok(Some(mixin))
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected mixin declaration; reached EOF instead."
            ),
        })
    }

    fn next_mixin_call(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        self.toker.skip_leading_whitespace();
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            let name = Borrowed(&self.toker.inner_str[name_beginning..name_end]);

            self.toker.offset = i;

            let arguments = if self.toker.eat("(").is_ok() {
                try!(self.tokenize_list(",", ")", &valid_mixin_arg_char))
            } else {
                Vec::new()
            };

            try!(self.toker.eat(";"));

            let mixin_call = TopLevelEvent::MixinCall(SassMixinCall {
                name: name,
                arguments: arguments,
            });

            return Ok(Some(mixin_call))

        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected mixin call; reached EOF instead."
            ),
        })
    }

    fn tokenize_list<F>(&mut self, separator: &str, end_list: &str, valid_char_fn: &F) -> Result<Vec<Cow<'a, str>>>
        where F: Fn(u8) -> bool {
        let mut list = Vec::new();

        let mut i = self.toker.offset;
        while i < self.limit() {
            self.toker.skip_leading_whitespace();
            i = self.toker.offset;
            let beginning = self.toker.offset;
            i += self.toker.scan_while_or_end(i, valid_char_fn);

            let n = scan_trailing_whitespace(&self.toker.inner_str[beginning..i]);
            let end = i - n;

            if end > beginning {
                list.push(Borrowed(&self.toker.inner_str[beginning..end]));
            }

            self.toker.offset = i;
            if self.toker.eat(end_list).is_ok() {
                break;
            } else {
                try!(self.toker.eat(separator));
            }
        }

        Ok(list)
    }

    fn selector_list(&mut self) -> Result<Vec<SassSelector<'a>>> {
        let selectors = try!(self.tokenize_list(",", "{", &valid_selector_char));
        debug!("selector list selectors = {:?}", selectors);
        self.state = State::InProperties;

        Ok(selectors.into_iter().map(|s| SassSelector::new(s)).collect())
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
