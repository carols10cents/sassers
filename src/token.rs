use operator_or_token::OperatorOrToken;
use operator::Operator;

use std::ops::{Add, Sub, Mul, Div, Rem};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OperatorOffset {
    pub operator: Operator,
    pub offset: Option<usize>,
}

impl fmt::Display for OperatorOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.operator.fmt(f)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(String),
    StringLiteral(String),
    Number { value: f32, units: Option<String>, computed: bool },
    Comment(String),
}

impl Add for Token {
    type Output = Token;

    fn add(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => {
                Token::Number {
                    value: self_value + other_value,
                    units: units,
                    computed: true,
                }
            },
            Err(msg) => panic!("Cannot add: {}", msg),
        }
    }
}

impl Sub for Token {
    type Output = Token;

    fn sub(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => {
                Token::Number {
                    value: self_value - other_value,
                    units: units,
                    computed: true,
                }
            },
            Err(msg) => panic!("Cannot subtract: {}", msg),
        }
    }
}

impl Mul for Token {
    type Output = Token;

    fn mul(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => {
                Token::Number {
                    value: self_value * other_value,
                    units: units,
                    computed: true,
                }
            },
            Err(msg) => panic!("Cannot multiply: {}", msg),
        }
    }
}

impl Div for Token {
    type Output = Token;

    fn div(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => {
                Token::Number {
                    value: self_value / other_value,
                    units: units,
                    computed: true,
                }
            },
            Err(msg) => panic!("Cannot divide: {}", msg),
        }
    }
}

impl Rem for Token {
    type Output = Token;

    fn rem(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => {
                Token::Number {
                    value: self_value % other_value,
                    units: units,
                    computed: true,
                }
            },
            Err(msg) => panic!("Cannot find the remainder: {}", msg),
        }
    }
}

fn mathy(first: Token, second: Token) -> Result<(f32, f32, Option<String>), String> {
    match (&first, &second) {
        (
            &Token::Number {
                value: ref first_value, units: ref first_units, ..
            },
            &Token::Number {
                value: ref second_value, units: ref _second_units, ..
            },
        ) => {
            Ok((*first_value, *second_value, first_units.clone()))
        },
        _ => Err(format!("Cannot perform math operations on {:?} and {:?}", first, second)),
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::String(ref i) => write!(f, "{}", i),
            Token::StringLiteral(ref i) => write!(f, "{}", i),
            Token::Number { value: i, units: Some(ref u), .. } => {
                write!(f, "{}{}", i, u)
            },
            Token::Number { value: i, units: None, .. } => write!(f, "{}", i),
            Token::Comment(ref i) => write!(f, "{}", i),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenOffset {
    pub token: Token,
    pub offset: Option<usize>,
}

impl From<OperatorOrToken> for TokenOffset {
    fn from(op_or_token: OperatorOrToken) -> TokenOffset {
        match op_or_token {
            OperatorOrToken::Token(t) => t,
            OperatorOrToken::Operator(OperatorOffset {
                operator: o, offset: off
            }) => TokenOffset {
                token: Token::String(o.to_string()),
                offset: off,
            }
        }
    }
}

impl fmt::Display for TokenOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.token.fmt(f)
    }
}

impl TokenOffset {
    pub fn combine(&self, other: &OperatorOrToken) -> TokenOffset {
        let self_string = self.token.to_string();
        let other_token = other.extract_token();
        let other_operator = other.extract_operator();

        let separator = if
            other_token == Some(Token::String(String::from("="))) ||
            other_token == Some(Token::String(String::from("]"))) ||
            self_string.ends_with("=") ||
            other_operator == Some(Operator::Star)
        {
            ""
        } else {
            " "
        };

        TokenOffset {
            token: Token::String(format!("{}{}{}", self, separator, other)),
            offset: self.offset,
        }
    }
}
