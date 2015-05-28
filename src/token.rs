pub use self::Token::*;
pub use self::DelimToken::*;

#[derive(Debug, Clone)]
pub struct Range {
    pub start_pos: u32,
    pub end_pos: u32,
}

#[derive(Debug, Clone)]
pub enum Token {
    Text(Range),
    Whitespace(Range),
    Semi(Range),
    Colon(Range),
    Comma(Range),
    OpenDelim(DelimToken, Range),
    CloseDelim(DelimToken, Range),
    Unknown(Range),
    Eof,
}

#[derive(Debug, Clone)]
pub enum DelimToken {
    Brace,
}
