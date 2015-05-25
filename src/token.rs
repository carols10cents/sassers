pub use self::Token::*;

#[derive(Debug, Clone)]
pub enum Token {
    Selector { start_pos: u32, end_pos: u32 },
    Eof,
}