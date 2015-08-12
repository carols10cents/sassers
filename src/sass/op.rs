use sass::value_part::ValuePart;

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

impl Op {
    pub fn apply<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        let (first_num, second_num) = match (first, second) {
            (ValuePart::Number(f), ValuePart::Number(s)) => (f, s),
            (f, s) => panic!("Invalid arguments {:?} {:?}", f, s),
        };
        let result = match *self {
            Op::Plus => first_num + second_num,
            Op::Minus => first_num - second_num,
            Op::Star => first_num * second_num,
            Op::Slash => first_num / second_num,
            Op::Percent => first_num % second_num,
            ref o => panic!("Invalid binary operator {:?}", o),
        };
        ValuePart::Number(result)
    }
}
