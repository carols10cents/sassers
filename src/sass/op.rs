use error::{Result, SassError, ErrorKind};
use sass::value_part::ValuePart;
use sass::color_value::ColorValue;
use evaluator::Evaluator;

use std::fmt;
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
    type Err = SassError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "+" => Ok(Op::Plus),
            "-" => Ok(Op::Minus),
            "*" => Ok(Op::Star),
            "/" => Ok(Op::Slash),
            "%" => Ok(Op::Percent),
            "(" => Ok(Op::LeftParen),
            ")" => Ok(Op::RightParen),
            "," => Ok(Op::Comma),
            op  => Err(SassError {
                offset: 0,
                kind: ErrorKind::InvalidOperator,
                message: format!("Invalid Operator: {}", op),
            }),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Op::Plus       => "+",
            Op::Minus      => "-",
            Op::Star       => "*",
            Op::Slash      => "/",
            Op::Percent    => "%",
            Op::LeftParen  => "(",
            Op::RightParen => ")",
            Op::Comma      => ",",
        };
        write!(f, "{}", s)
    }
}


impl Op {
    pub fn apply(&self, first: ValuePart, second: ValuePart, paren_level: i32) -> Result<ValuePart> {
        match (self, second) {
            (&Op::Plus, s @ ValuePart::List(..)) => self.apply_list(first, s),
            (&Op::Plus, s @ ValuePart::String(..)) => {
                Ok(ValuePart::String(format!("{}{}", first, s).into()))
            },
            (&Op::Slash, s) => self.apply_slash(first, s, paren_level),
            (&Op::Comma, s) => {
                ValuePart::concat_into_list(
                    try!(ValuePart::concat_into_list(first, ValuePart::Operator(*self))),
                    s,
                )
            },
            (_, s) => {
                self.apply_math(first, s)
            },
        }
    }

    fn force_list_collapse(&self, mut vp: ValuePart) -> Result<ValuePart> {
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
                    Evaluator::new(
                        ve.into_iter().map(|v| Ok(v)).collect::<Vec<_>>()
                    ).evaluate(&HashMap::new())
                } else {
                    Ok(ValuePart::List(l.clone()))
                }
            },
            _ => Ok(vp),
        }
    }

    fn apply_list(&self, first: ValuePart, second: ValuePart) -> Result<ValuePart> {
        let first_collapsed  = try!(self.force_list_collapse(first));
        let second_collapsed = try!(self.force_list_collapse(second));

        match (first_collapsed, second_collapsed) {
            (ValuePart::Number(fnum), ValuePart::List(mut slist)) => {
                let new_first_item_value = format!("{}{}", fnum, slist.remove(0));
                let v = ValuePart::String(new_first_item_value.into());

                ValuePart::concat_into_list(v, ValuePart::List(slist))
            },
            (ValuePart::List(mut flist), ValuePart::Number(snum)) => {
                let new_last_item_value = format!("{}{}",
                    flist.pop().unwrap_or(ValuePart::String(String::from(""))),
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
                Err(SassError {
                    offset: 0,
                    kind: ErrorKind::InvalidApplyListArgs,
                    message: format!(
                        "Invalid apply_list arguments:\n  first: {:?}\n  second: {:?}",
                        unk_first, unk_second
                    ),
                })
            },
        }
    }

    fn apply_slash(&self, first: ValuePart, second: ValuePart, paren_level: i32) -> Result<ValuePart> {
        if paren_level == 0 {
            if first.computed_number() || second.computed_number() {
                let first_collapsed  = try!(self.force_list_collapse(first));
                let second_collapsed = try!(self.force_list_collapse(second));

                self.apply_math(first_collapsed, second_collapsed)
            } else {
                ValuePart::concat_into_list(
                    try!(ValuePart::concat_into_list(first, ValuePart::Operator(*self))),
                    second,
                )
            }
        } else {
            match first {
                ValuePart::List(..) => {
                    ValuePart::concat_into_list(
                        try!(ValuePart::concat_into_list(first, ValuePart::Operator(*self))),
                        second,
                    )
                },
                _ => {
                    self.apply_math(first, second)
                },
            }
        }
    }

    fn apply_math(&self, first: ValuePart, second: ValuePart) -> Result<ValuePart> {
        match (first, second) {
            (ValuePart::Number(f), ValuePart::Number(s)) => {
                Ok(ValuePart::Number(try!(f.apply_math(*self, s))))
            },
            (ValuePart::Color(f), ValuePart::Number(s)) => {
                Ok(ValuePart::Color(try!(f.apply_math(*self, s))))
            },
            (ValuePart::Color(f), ValuePart::Color(s)) => {
                Ok(ValuePart::Color(try!(f.combine_colors(*self, s))))
            },
            (ValuePart::Number(f), ValuePart::Color(s)) => {
                ValuePart::concat_into_list(
                    try!(ValuePart::concat_into_list(
                        ValuePart::Number(f), ValuePart::Operator(*self)
                    )),
                    ValuePart::Color(ColorValue { computed: true, ..s}),
                )
            },
            (f, s) => {
                Err(SassError {
                    offset: 0,
                    kind: ErrorKind::InvalidApplyMathArgs,
                    message: format!(
                        "Invalid apply math arguments:\n  first: {:?}\n  second: {:?}",
                        f, s
                    ),
                })
            },
        }
    }

    pub fn math(&self, first_num: f32, second_num: f32) -> Result<f32> {
        let res = match *self {
            Op::Plus    => first_num + second_num,
            Op::Minus   => first_num - second_num,
            Op::Star    => first_num * second_num,
            Op::Slash   => first_num / second_num,
            Op::Percent => first_num % second_num,
            other => return Err(SassError {
                offset: 0,
                kind: ErrorKind::InvalidOperator,
                message: format!(
                    "Cannot apply operator {:?} as math",
                    other
                ),
            }),
        };
        Ok(res)
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
