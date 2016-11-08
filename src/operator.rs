use std::fmt;

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
