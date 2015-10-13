use error::{SassError, ErrorKind,Result};
use event::Event;
use sass::comment::SassComment;
use sass::variable::SassVariable;
use top_level_event::TopLevelEvent;
use tokenizer_utils::*;
use inner_tokenizer::{InnerTokenizer, State};

#[derive(Debug)]
pub struct Tokenizer<'a> {
    toker: Toker<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(inner_str: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            toker: Toker {
                inner_str: &inner_str,
                bytes: &inner_str.as_bytes(),
                offset: 0,
            },
        }
    }

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn parse(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'$' {
            return self.next_variable()
        }

        if self.toker.eat("@mixin ").is_ok() {
            return self.next_mixin()
        }

        if self.toker.eat("@include ").is_ok() {
            return self.next_mixin_call()
        }

        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                return self.next_comment()
            }
        }

        let mut inner = InnerTokenizer {
            toker: Toker {
                inner_str: &self.toker.inner_str,
                bytes: &self.toker.bytes,
                offset: self.toker.offset,
            },
            state: State::InSelectors,
        };
        let ret = match inner.next_rule() {
            Ok(Some(Event::ChildRule(rule))) => Ok(Some(TopLevelEvent::Rule(rule))),
            other => return Err(SassError {
                kind: ErrorKind::TokenizerError,
                message: format!(
                    "Expected sass rule from inner tokenizer, got: {:?}.",
                    other
                ),
            }),
        };
        self.toker.offset = inner.toker.offset;
        return ret
    }

    fn next_comment(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        match self.toker.next_comment() {
            Ok(Some(Event::Comment(comment))) => {
                Ok(Some(TopLevelEvent::Comment(SassComment { comment: comment })))
            },
            Err(e) => Err(e),
            _ => unreachable!(),
        }
    }

    fn next_variable(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let var_name = try!(self.toker.next_name());

        try!(self.toker.eat(":"));
        self.toker.skip_leading_whitespace();

        let var_value = try!(self.toker.next_value());

        try!(self.toker.eat(";"));
        self.toker.skip_leading_whitespace();

        Ok(Some(TopLevelEvent::Variable(SassVariable {
            name:  var_name,
            value: var_value,
        })))
    }

    fn next_mixin(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        match self.toker.next_mixin() {
            Ok(Some(Event::Mixin(sass_mixin))) => {
                 Ok(Some(TopLevelEvent::Mixin(sass_mixin)))
            },
            Err(e) => Err(e),
            _ => unreachable!(),
        }
    }

    fn next_mixin_call(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        match self.toker.next_mixin_call() {
            Ok(Some(Event::MixinCall(sass_mixin_call))) => {
                 Ok(Some(TopLevelEvent::MixinCall(sass_mixin_call)))
            },
            Err(e) => Err(e),
            _ => unreachable!(),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<TopLevelEvent<'a>>;

    fn next(&mut self) -> Option<Result<TopLevelEvent<'a>>> {
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
