pub use self::Token::*;
pub use self::DelimToken::*;

#[derive(Debug, Clone)]
pub struct Range {
    pub start_pos: u32,
    pub end_pos: u32,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub enum Token {
    Text,
    Whitespace,
    Semi,
    Colon,
    Comma,
    OpenDelim(DelimToken),
    CloseDelim(DelimToken),
    Unknown,
    Eof,
}

#[derive(Debug, Clone)]
pub enum DelimToken {
    Brace,
}
