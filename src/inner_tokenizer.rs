use error::Result;
use event::Event;
use sass::selector::SassSelector;
use sass::variable::SassVariable;
use sass::rule::SassRule;
use tokenizer_utils::*;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum State {
    InComment,
    InSelectors,
    InProperties,
}

#[derive(Debug)]
pub struct InnerTokenizer<'a> {
    pub toker: Toker<'a>,
    pub state: State,
}

impl<'a> InnerTokenizer<'a> {

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn parse(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'}' {
            return Ok(None)
        }
        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                return self.toker.next_comment()
            }
        }

        match self.state {
            State::InProperties => self.next_property(),
            State::InSelectors => self.next_rule(),
            other => unreachable!("got {:?}", other),
        }
    }

    pub fn next_rule(&mut self) -> Result<Option<Event<'a>>> {
        debug!("in next rule, offset {:?}", self.toker.offset);
        let mut current_sass_rule = SassRule::new();
        current_sass_rule.selectors = try!(self.selector_list());
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
        debug!("returned from inner rule, offset {:?}", self.toker.offset);
        self.state = State::InProperties;

        while let Some(Ok(e)) = self.next() {
            current_sass_rule.children.push(e);
        }
        try!(self.toker.eat("}"));

        Ok(Some(Event::ChildRule(current_sass_rule)))
    }

    fn next_property(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.curr_byte();
        if c == b'}' {
            return Ok(None)
        }

        let d = self.toker.next_byte();
        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return Ok(None)
        }

        let saved_offset = self.toker.offset;

        if self.toker.eat("@include ").is_ok() {
            return self.toker.next_mixin_call()
        }

        if self.toker.eat("@extend ").is_ok() {
            return self.toker.next_mixin_call()
        }

        let prop_name = try!(self.toker.next_name());

        let c = self.toker.curr_byte();
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

        let prop_value = try!(self.toker.next_value());

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

    fn selector_list(&mut self) -> Result<Vec<SassSelector<'a>>> {
        let selectors = try!(self.toker.tokenize_list(",", "{", &valid_selector_char));
        self.state = State::InProperties;

        Ok(selectors.into_iter().map(|s| SassSelector::new(s)).collect())
    }
}

impl<'a> Iterator for InnerTokenizer<'a> {
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Result<Event<'a>>> {
        if !self.toker.at_eof() {
            return match self.parse() {
                Ok(Some(t)) => Some(Ok(t)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        None
    }
}
