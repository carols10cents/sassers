use token::{Lexeme, Token};

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

    pub fn extract_scalar(&self) -> f32 {
        match self.scalar.token {
            Token::Number(num, _) => num,
            _ => panic!("Had a non-numeric Token in a NumberValue!!!"),
        }
    }

    pub fn apply_math(&self, operator: &Lexeme, other: &NumberValue) -> NumberValue {
        let f = self.extract_scalar();
        let s = other.extract_scalar();

        let result = match operator.token {
            Token::Plus    => f + s,
            Token::Minus   => f - s,
            Token::Star    => f * s,
            Token::Slash   => f / s,
            Token::Percent => f % s,
            _ => unimplemented!(),
        };

        NumberValue {
            scalar: Lexeme {
                token: Token::Number(result, None),
                offset: self.offset(),
            },
            computed: true,
        }
    }

}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.scalar.token)
    }
}
