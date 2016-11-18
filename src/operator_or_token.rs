use token::Token;
use token_offset::TokenOffset;
use operator::Operator;
use operator_offset::OperatorOffset;

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

    pub fn computed_number(&self) -> bool {
        match *self {
            OperatorOrToken::Token(TokenOffset {
                token: Token::Number { computed: true, .. }, ..
            }) => true,
            _ => false,
        }
    }

    pub fn extract_token(&self) -> Option<Token> {
        if let OperatorOrToken::Token(ref t) = *self {
            Some(t.token.clone())
        } else {
            None
        }
    }

    pub fn extract_operator(&self) -> Option<Operator> {
        if let OperatorOrToken::Operator(o) = *self {
            Some(o.operator)
        } else {
            None
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
