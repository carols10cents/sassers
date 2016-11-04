use token::{Lexeme, Token};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue {
    pub offset:   Option<usize>,
    pub scalar:   f32,
    pub units:    Option<String>,
    pub computed: bool,
}

impl<'b> NumberValue {
    pub fn from_scalar(num: Lexeme) -> NumberValue {
        match num {
            Lexeme { token: Token::Number(scalar, units), offset: o } => {
                NumberValue {
                    offset:   o,
                    scalar:   scalar,
                    units:    units,
                    computed: false,
                }
            },
            _ => panic!("Had a non-numeric Token in a NumberValue!!!"),
        }
    }

    pub fn apply_math(&self, operator: &Lexeme, other: &NumberValue) -> NumberValue {
        let f = self.scalar;
        let s = other.scalar;

        let result = match operator.token {
            Token::Plus    => f + s,
            Token::Minus   => f - s,
            Token::Star    => f * s,
            Token::Slash   => f / s,
            Token::Percent => f % s,
            _ => unimplemented!(),
        };

        NumberValue {
            offset:   self.offset,
            scalar:   result,
            units:    None,
            computed: true,
        }
    }
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.units {
            Some(ref u) => write!(f, "{}{}", self.scalar, u),
            None => write!(f, "{}", self.scalar),
        }
    }
}
