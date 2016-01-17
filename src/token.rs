use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Lexeme {
    pub token: Token,
    pub offset: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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

    Ident(String),
    Number(f32),
    // Variable(String),
    // Literal(String),
    // Comment(String),
    // Color(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::LeftCurlyBrace => write!(f, "{{"),
            Token::RightCurlyBrace => write!(f, "}}"),

            Token::Ident(ref i) => write!(f, "{}", i),
            Token::Number(i) => write!(f, "{}", i),
            // Token::Variable(ref i) => write!(f, "{}", i),
            // Token::Literal(ref i) => write!(f, "{}", i),
            // Token::Comment(ref i) => write!(f, "{}", i),
            // Token::Color(ref i) => write!(f, "{}", i),
        }
    }
}

impl Token {
    pub fn from_char(c: char) -> Option<Token> {
        let r = match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '{' => Token::LeftCurlyBrace,
            '}' => Token::RightCurlyBrace,
            _   => return None,
        };
        Some(r)
    }
}
