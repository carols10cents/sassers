use sass::value_part::ValuePart;

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn apply<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>, paren_level: i32) -> ValuePart<'a> {
        match (self, &second) {
            (&Op::Plus, &ValuePart::List(..)) => self.apply_list(first, second),
            (&Op::Slash, _) => self.apply_slash(first, second, paren_level),
            (_, _) => self.apply_math(first, second),
        }
    }

    fn apply_list<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        match (first, second) {
            (ValuePart::Number(f), ValuePart::List(mut l)) => {
                let new_first_item_value = format!("{}{}", f, l.remove(0));
                let v = ValuePart::String(new_first_item_value.into());
                let mut ve = vec![v];
                ve.append(&mut l);
                ValuePart::List(ve)
            },
            (_, _) => ValuePart::List(vec![]),
        }
    }

    fn apply_slash<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>, paren_level: i32) -> ValuePart<'a> {
        if paren_level == 0 {
            ValuePart::List(vec![first, ValuePart::Operator(*self), second])
        } else {
            self.apply_math(first, second)
        }
    }

    fn apply_math<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        let (first_num, second_num) = match (first, second) {
            (ValuePart::Number(f), ValuePart::Number(s)) => (f, s),
            (f, s) => return ValuePart::String(format!("Invalid arguments {:?} {:?}", f, s).into()),
        };
        let result = match *self {
            Op::Plus => first_num + second_num,
            Op::Minus => first_num - second_num,
            Op::Star => first_num * second_num,
            Op::Slash => first_num / second_num,
            Op::Percent => first_num % second_num,
            _ => 0.0,
        };
        ValuePart::Number(result)
    }

    pub fn same_or_greater_precedence(self, other: Op) -> bool {
        match (self, other) {
            (Op::Plus, Op::Star) |
            (Op::Minus, Op::Star) |
            (Op::Plus, Op::Slash) |
            (Op::Minus, Op::Slash) |
            (Op::LeftParen, _) => false,
            (_, _) => true,
        }
    }
}
