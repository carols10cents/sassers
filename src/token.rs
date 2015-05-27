pub use self::Token::*;
pub use self::DelimToken::*;

#[derive(Debug, Clone)]
pub enum Token {
    Selector { start_pos: u32, end_pos: u32 },
    Property { start_pos: u32, end_pos: u32 },
    Value { start_pos: u32, end_pos: u32 },
    Whitespace { start_pos: u32, end_pos: u32 },
    Semi,
    Colon,
    Comma,
    OpenDelim(DelimToken),
    CloseDelim(DelimToken),
    Eof,
    Unknown { pos: u32 },
}

#[derive(Debug, Clone)]
pub enum DelimToken {
    Brace,
}
