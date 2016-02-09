use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Lexeme {
    pub token: Token,
    pub offset: Option<usize>,
}

impl Lexeme {
    pub fn new() -> Lexeme {
        Lexeme {
            token: Token::String("".into()),
            offset: None,
        }
    }

    pub fn combine(&self, other: &Lexeme) -> Lexeme {
        let offset = match self.offset {
            Some(o) => Some(o),
            None => other.offset,
        };
        Lexeme {
            token: self.token.combine(&other.token),
            offset: offset,
        }
    }
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

    String(String),
    StringLiteral(String),

    Number(f32, Option<String>),

    // Comment(String),
}

impl Token {
    pub fn combine(&self, other: &Token) -> Token {
        match (self, other) {
            (&Token::String(ref my_str), &Token::String(ref other_str)) => {
                if my_str.len() > 0 {
                    Token::String(format!("{} {}", my_str, other_str))
                } else {
                    Token::String(other_str.clone())
                }
            },
            (s, o) => panic!("Don't know how to combine {:?} and {:?}", s, o),
        }
    }
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

            Token::String(ref i) => write!(f, "{}", i),
            Token::StringLiteral(ref i) => write!(f, "{}", i),
            Token::Number(i, Some(ref u)) => write!(f, "{}{}", i, u),
            Token::Number(i, None) => write!(f, "{}", i),
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
