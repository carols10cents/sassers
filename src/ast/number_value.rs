use token::{Lexeme};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue {
    pub scalar:   Lexeme,
    pub computed: bool,
}

impl<'b> NumberValue {
    pub fn offset(&self) -> Option<usize> {
        self.scalar.offset
    }

    pub fn from_scalar(num: Lexeme) -> NumberValue {
        NumberValue {
            scalar:   num,
            computed: false,
        }
    }
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.scalar.token)
    }
}
