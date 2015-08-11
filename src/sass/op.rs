use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LeftParen,
    RightParen,
    Comma,
}

impl FromStr for Op {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Op::Plus),
            "-" => Ok(Op::Minus),
            "*" => Ok(Op::Star),
            "/" => Ok(Op::Slash),
            "%" => Ok(Op::Percent),
            "(" => Ok(Op::LeftParen),
            ")" => Ok(Op::RightParen),
            "," => Ok(Op::Comma),
            _   => Err(()),
        }
    }
}
