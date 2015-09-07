use sass::value_part::ValuePart;
use sass::number_value::NumberValue;
use evaluator::Evaluator;

use std::borrow::Cow::Borrowed;
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

    fn force_list_collapse<'a>(&self, mut vp: ValuePart<'a>) -> ValuePart<'a> {
        match vp {
            ValuePart::List(ref mut l) => {
                let mut ve = vec![ValuePart::Operator(Op::LeftParen)];
                l.push(ValuePart::Operator(Op::RightParen));
                ve.append(l);
                Evaluator::new(ve).evaluate()
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
                let mut ve = vec![v];
                ve.append(&mut slist);
                ValuePart::List(ve)
            },
            (ValuePart::List(mut flist), ValuePart::Number(snum)) => {
                let new_last_item_value = format!("{}{}",
                    flist.pop().unwrap_or(ValuePart::String(Borrowed(""))),
                    snum
                );
                let v = ValuePart::String(new_last_item_value.into());
                flist.push(v);
                ValuePart::List(flist)
            },
            (ValuePart::List(mut flist), ValuePart::List(slist)) => {
                flist.extend(slist);
                ValuePart::List(flist.clone())
            },
            (f @ ValuePart::Number(..), s @ ValuePart::Number(..)) => {
                self.apply_math(f, s)
            },
            (unk_first, unk_second) => {
                panic!(
                    "Unknown apply_list match:\n  first: {:?}\n  second: {:?}",
                    unk_first, unk_second
                )
            },
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
                ValuePart::Number(ref n) if n.computed => {
                    self.apply_math(ValuePart::Number(n.clone()), second)
                },
                _ => ValuePart::List(vec![first, ValuePart::Operator(*self), second]),
            }
        } else {
            self.apply_math(first, second)
        }
    }

    fn apply_math<'a>(&self, first: ValuePart<'a>, second: ValuePart<'a>) -> ValuePart<'a> {
        let (first_num, second_num) = match (first, second) {
            (ValuePart::Number(f), ValuePart::Number(s)) => (f.scalar, s.scalar),
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
        ValuePart::Number(NumberValue::computed(result))
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
