use sass::op::Op;

use std::borrow::Cow::*;
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue<'a> {
    pub scalar:   f32,
    pub unit:     Option<Cow<'a, str>>,
    pub computed: bool,
}

impl<'a, 'b> NumberValue<'a> {
    pub fn from_scalar(num: f32) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: false,
        }
    }

    #[cfg(test)]
    pub fn computed(num: f32) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: true,
        }
    }

    pub fn with_units(num: f32, unit: Cow<'a, str>) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     Some(unit),
            computed: false,
        }
    }

    pub fn into_owned(self) -> NumberValue<'b> {
        NumberValue {
            scalar: self.scalar,
            unit: match self.unit {
                Some(u) => Some(u.into_owned().into()),
                None => None,
            },
            computed: self.computed,
        }
    }

    pub fn unit_string(&self) -> &str {
        match self.unit {
            Some(Borrowed(ref b)) => b,
            Some(Owned(ref o))    => o,
            None                  => "",
        }
    }

    pub fn apply_math(self, op: Op, nv: NumberValue<'a>) -> NumberValue<'a> {
        let result       = self.compute_number(op, &nv);
        let result_units = self.compute_units(op, nv);

        NumberValue {
            scalar:   result,
            unit:     result_units,
            computed: true,
        }
    }

    fn compute_number(&self, op: Op, nv: &NumberValue<'a>) -> f32 {
        let first_num = self.scalar;
        let second_num = nv.scalar;
        match op {
            Op::Plus    => first_num + second_num,
            Op::Minus   => first_num - second_num,
            Op::Star    => first_num * second_num,
            Op::Slash   => first_num / second_num,
            Op::Percent => first_num % second_num,
            _           => 0.0, // TODO: Result
        }
    }

    fn compute_units(self, op: Op, nv: NumberValue<'a>) -> Option<Cow<'a, str>> {
        match (self.unit, nv.unit) {
            (Some(u), None) | (None, Some(u)) => Some(u.into_owned().into()),
            (Some(ref u1), Some(ref u2)) if u1 == u2 => {
                match op {
                    Op::Slash => None, // Divide out the units
                    // TODO: Multiplication that produces square units should return Err
                    _         => Some(u1.clone().into_owned().into()),
                }
            },
            (None, None) => None,
            (other1, other2) => {
                Some(Owned(format!("Incompatible units: {:?} and {:?}", other1, other2))) // TODO: Result
            },
        }
    }
}

impl<'a> fmt::Display for NumberValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.scalar, self.unit_string())
    }
}
