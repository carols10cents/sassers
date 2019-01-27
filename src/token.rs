use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(String),
    StringLiteral(String),
    Number {
        value: f32,
        units: Option<String>,
        computed: bool,
    },
    Comment(String),
}

impl Add for Token {
    type Output = Token;

    fn add(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => Token::Number {
                value: self_value + other_value,
                units: units,
                computed: true,
            },
            Err(msg) => panic!("Cannot add: {}", msg),
        }
    }
}

impl Sub for Token {
    type Output = Token;

    fn sub(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => Token::Number {
                value: self_value - other_value,
                units: units,
                computed: true,
            },
            Err(msg) => panic!("Cannot subtract: {}", msg),
        }
    }
}

impl Mul for Token {
    type Output = Token;

    fn mul(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => Token::Number {
                value: self_value * other_value,
                units: units,
                computed: true,
            },
            Err(msg) => panic!("Cannot multiply: {}", msg),
        }
    }
}

impl Div for Token {
    type Output = Token;

    fn div(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => Token::Number {
                value: self_value / other_value,
                units: units,
                computed: true,
            },
            Err(msg) => panic!("Cannot divide: {}", msg),
        }
    }
}

impl Rem for Token {
    type Output = Token;

    fn rem(self, other: Token) -> Token {
        match mathy(self, other) {
            Ok((self_value, other_value, units)) => Token::Number {
                value: self_value % other_value,
                units: units,
                computed: true,
            },
            Err(msg) => panic!("Cannot find the remainder: {}", msg),
        }
    }
}

fn mathy(first: Token, second: Token) -> Result<(f32, f32, Option<String>), String> {
    match (&first, &second) {
        (
            &Token::Number {
                value: ref first_value,
                units: ref first_units,
                ..
            },
            &Token::Number {
                value: ref second_value,
                units: ref _second_units,
                ..
            },
        ) => Ok((*first_value, *second_value, first_units.clone())),
        _ => Err(format!(
            "Cannot perform math operations on {:?} and {:?}",
            first, second
        )),
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::String(ref i) => write!(f, "{}", i),
            Token::StringLiteral(ref i) => write!(f, "{}", i),
            Token::Number {
                value: i,
                units: Some(ref u),
                ..
            } => write!(f, "{}{}", i, u),
            Token::Number {
                value: i,
                units: None,
                ..
            } => write!(f, "{}", i),
            Token::Comment(ref i) => write!(f, "{}", i),
        }
    }
}
