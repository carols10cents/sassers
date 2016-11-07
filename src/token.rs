use std::ops::{Add, Sub, Mul, Div, Rem};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum OperatorOrToken {
    Operator(OperatorOffset),
    Token(TokenOffset),
}

impl OperatorOrToken {
    pub fn offset(&self) -> Option<usize> {
        match *self {
            OperatorOrToken::Operator(o) => o.offset,
            OperatorOrToken::Token(ref t) => t.offset,
        }
    }
}

impl Add for OperatorOrToken {
    type Output = OperatorOrToken;

    fn add(self, other: OperatorOrToken) -> OperatorOrToken {
        match (self, other) {
            (
                OperatorOrToken::Token(TokenOffset {
                    token: self_token,
                    offset: off,
                }),
                OperatorOrToken::Token(TokenOffset {
                    token: other_token, ..
                })
            ) => {
                OperatorOrToken::Token(TokenOffset {
                    token: self_token + other_token,
                    offset: off,
                })
            },
            (s, other) => panic!("Cannot add: {:?} + {:?}", s, other),
        }
    }
}

impl Sub for OperatorOrToken {
    type Output = OperatorOrToken;

    fn sub(self, other: OperatorOrToken) -> OperatorOrToken {
        match (self, other) {
            (
                OperatorOrToken::Token(TokenOffset {
                    token: self_token,
                    offset: off,
                }),
                OperatorOrToken::Token(TokenOffset {
                    token: other_token, ..
                })
            ) => {
                OperatorOrToken::Token(TokenOffset {
                    token: self_token - other_token,
                    offset: off,
                })
            },
            (s, other) => panic!("Cannot subtract: {:?} - {:?}", s, other),
        }
    }
}

impl Mul for OperatorOrToken {
    type Output = OperatorOrToken;

    fn mul(self, other: OperatorOrToken) -> OperatorOrToken {
        match (self, other) {
            (
                OperatorOrToken::Token(TokenOffset {
                    token: self_token,
                    offset: off,
                }),
                OperatorOrToken::Token(TokenOffset {
                    token: other_token, ..
                })
            ) => {
                OperatorOrToken::Token(TokenOffset {
                    token: self_token * other_token,
                    offset: off,
                })
            },
            (s, other) => panic!("Cannot multiply: {:?} * {:?}", s, other),
        }
    }
}

impl Div for OperatorOrToken {
    type Output = OperatorOrToken;

    fn div(self, other: OperatorOrToken) -> OperatorOrToken {
        match (self, other) {
            (
                OperatorOrToken::Token(TokenOffset {
                    token: self_token,
                    offset: off,
                }),
                OperatorOrToken::Token(TokenOffset {
                    token: other_token, ..
                })
            ) => {
                OperatorOrToken::Token(TokenOffset {
                    token: self_token / other_token,
                    offset: off,
                })
            },
            (s, other) => panic!("Cannot divide: {:?} / {:?}", s, other),
        }
    }
}

impl Rem for OperatorOrToken {
    type Output = OperatorOrToken;

    fn rem(self, other: OperatorOrToken) -> OperatorOrToken {
        match (self, other) {
            (
                OperatorOrToken::Token(TokenOffset {
                    token: self_token,
                    offset: off,
                }),
                OperatorOrToken::Token(TokenOffset {
                    token: other_token, ..
                })
            ) => {
                OperatorOrToken::Token(TokenOffset {
                    token: self_token % other_token,
                    offset: off,
                })
            },
            (s, other) => panic!("Cannot find the remainder: {:?} % {:?}", s, other),
        }
    }
}

impl fmt::Display for OperatorOrToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OperatorOrToken::Operator(o) => o.fmt(f),
            OperatorOrToken::Token(ref t) => t.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LeftParen,
    RightParen,
    Comma,
    Colon,
    Semicolon,
    LeftCurlyBrace,
    RightCurlyBrace,
}

impl Operator {
    pub fn from_char(c: char) -> Option<Operator> {
        let r = match c {
            '+' => Operator::Plus,
            '-' => Operator::Minus,
            '*' => Operator::Star,
            '/' => Operator::Slash,
            '%' => Operator::Percent,
            '(' => Operator::LeftParen,
            ')' => Operator::RightParen,
            ',' => Operator::Comma,
            ':' => Operator::Colon,
            ';' => Operator::Semicolon,
            '{' => Operator::LeftCurlyBrace,
            '}' => Operator::RightCurlyBrace,
            _   => return None,
        };
        Some(r)
    }
}

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

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Star => write!(f, "*"),
            Operator::Slash => write!(f, "/"),
            Operator::Percent => write!(f, "%"),
            Operator::LeftParen => write!(f, "("),
            Operator::RightParen => write!(f, ")"),
            Operator::Comma => write!(f, ","),
            Operator::Colon => write!(f, ":"),
            Operator::Semicolon => write!(f, ";"),
            Operator::LeftCurlyBrace => write!(f, "{{"),
            Operator::RightCurlyBrace => write!(f, "}}"),
        }
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
        let separator = match *other {
            OperatorOrToken::Token(TokenOffset { ref token, .. }) => {
                match token {
                    &Token::String(ref s) => {
                        if *s == String::from("=") ||
                           *s == String::from("]") ||
                           (*s).ends_with("=") {
                            ""
                        } else {
                            " "
                        }
                    },
                    _ => " ",
                }
            },
            OperatorOrToken::Operator(
                OperatorOffset { operator: Operator::Star, ..}) => "",
            _ => " ",
        };

        TokenOffset {
            token: Token::String(format!("{}{}{}", self, separator, other)),
            offset: self.offset,
        }
    }
}
