use sass::value_part::ValuePart;
use evaluator::Evaluator;

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

    fn apply_list<'a>(&self, mut first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        let mut l = match second {
            ValuePart::List(contents) => contents,
            _ => unreachable!(), // only calling this on lists i swear
        };
        match first {
            ValuePart::Number(f) => {
                if l.iter().any(|item| *item == ValuePart::Operator(Op::Slash)) {
                    let mut ve = vec![ValuePart::Operator(Op::LeftParen)];
                    l.push(ValuePart::Operator(Op::RightParen));
                    ve.append(&mut l);
                    self.apply_math(ValuePart::Number(f), Evaluator::new(ve).evaluate())
                } else {
                    let new_first_item_value = format!("{}{}", f, l.remove(0));
                    let v = ValuePart::String(new_first_item_value.into());
                    let mut ve = vec![v];
                    ve.append(&mut l);
                    ValuePart::List(ve)
                }
            },
            ValuePart::List(ref mut f) => {
                f.extend(l);
                ValuePart::List(f.clone())
            },
            _ => ValuePart::List(vec![]),
        }
    }

    fn apply_slash<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>, paren_level: i32) -> ValuePart<'a> {
        if paren_level == 0 {
            match first {
                ValuePart::List(mut f) => {
                    let mut ve = vec![ValuePart::Operator(*self), second];
                    f.append(&mut ve);
                    ValuePart::List(f)
                },
                ValuePart::Computed(..) => {
                    self.apply_math(first, second)
                },
                _ => ValuePart::List(vec![first, ValuePart::Operator(*self), second]),
            }
        } else {
            self.apply_math(first, second)
        }
    }

    fn apply_math<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        let (first_num, second_num) = match (first, second) {
            (ValuePart::Number(f), ValuePart::Number(s)) => (f, s),
            (ValuePart::Number(f), ValuePart::Computed(s)) => (f, s),
            (ValuePart::Computed(f), ValuePart::Number(s)) => (f, s),
            (ValuePart::Computed(f), ValuePart::Computed(s)) => (f, s),
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
        ValuePart::Computed(result)
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
