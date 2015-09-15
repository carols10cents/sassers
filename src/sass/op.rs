use sass::value_part::ValuePart;
use evaluator::Evaluator;

use std::borrow::Cow::*;
use std::str::FromStr;
use std::collections::HashMap;

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
        match (self, second) {
            (&Op::Plus, s @ ValuePart::List(..)) => self.apply_list(first, s),
            (&Op::Plus, s @ ValuePart::String(..)) => {
                ValuePart::String(format!("{}{}", first, s).into())
            },
            (&Op::Slash, s) => self.apply_slash(first, s, paren_level),
            (&Op::Comma, s) => {
                ValuePart::concat_into_list(
                    ValuePart::concat_into_list(first, ValuePart::Operator(*self)),
                    s,
                )
            },
            (_, s) => {
                self.apply_math(first, s)
            },
        }
    }

    fn force_list_collapse<'a>(&self, mut vp: ValuePart<'a>) -> ValuePart<'a> {
        match vp {
            ValuePart::List(ref mut l) => {
                if l.iter().any(|v| {
                    match v {
                        &ValuePart::Operator(Op::Slash) => true,
                        _ => false,
                    }
                }) {
                    let mut ve = vec![ValuePart::Operator(Op::LeftParen)];
                    l.push(ValuePart::Operator(Op::RightParen));
                    ve.append(l);
                    Evaluator::new(ve).evaluate(&HashMap::new())
                } else {
                    ValuePart::List(l.clone())
                }
            },
            _ => vp,
        }
    }

    fn apply_list<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        let first_collapsed  = self.force_list_collapse(first);
        let second_collapsed = self.force_list_collapse(second);

        match (first_collapsed, second_collapsed) {
            (ValuePart::Number(fnum), ValuePart::List(mut slist)) => {
                let new_first_item_value = format!("{}{}", fnum, slist.remove(0));
                let v = ValuePart::String(new_first_item_value.into());

                ValuePart::concat_into_list(v, ValuePart::List(slist))
            },
            (ValuePart::List(mut flist), ValuePart::Number(snum)) => {
                let new_last_item_value = format!("{}{}",
                    flist.pop().unwrap_or(ValuePart::String(Borrowed(""))),
                    snum
                );
                let v = ValuePart::String(new_last_item_value.into());
                ValuePart::concat_into_list(ValuePart::List(flist), v)
            },
            (f @ ValuePart::List(..), s @ ValuePart::List(..)) => {
                ValuePart::concat_into_list(f, s)
            },
            (f @ ValuePart::Number(..), s @ ValuePart::Number(..)) => {
                self.apply_math(f, s)
            },
            (unk_first, unk_second) => {
                return ValuePart::String(format!(
                    "Unknown apply_list match:\n  first: {:?}\n  second: {:?}",
                    unk_first, unk_second
                ).into())
            },
        }
    }

    fn apply_slash<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>, paren_level: i32) -> ValuePart<'a> {
        if paren_level == 0 {
            if first.computed_number() || second.computed_number() {
                self.apply_math(self.force_list_collapse(first), self.force_list_collapse(second))
            } else {
                ValuePart::concat_into_list(
                    ValuePart::concat_into_list(first, ValuePart::Operator(*self)),
                    second,
                )
            }
        } else {
            match first {
                ValuePart::List(..) => {
                    ValuePart::concat_into_list(
                        ValuePart::concat_into_list(first, ValuePart::Operator(*self)),
                        second,
                    )
                },
                _ => {
                    self.apply_math(first, second)
                },
            }
        }
    }

    fn apply_math<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        match (first, second) {
            (ValuePart::Number(f), ValuePart::Number(s)) => {
                ValuePart::Number(f.apply_math(*self, s))
            },
            (f, s) => {
                // TODO: result
                ValuePart::String(
                    format!("Invalid apply math arguments:\n  first: {:?}\n  second: {:?}\n", f, s).into()
                )
            },
        }
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
