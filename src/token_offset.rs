use token::Token;
use operator_or_token::OperatorOrToken;
use operator_offset::OperatorOffset;
use operator::Operator;

use std::fmt;

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
