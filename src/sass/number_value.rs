use sass::op::Op;
use error::{Result, SassError, ErrorKind};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue {
    pub scalar:   f32,
    pub unit:     Option<String>,
    pub computed: bool,
}

impl<'b> NumberValue {
    pub fn from_scalar(num: f32) -> NumberValue {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: false,
        }
    }

    #[cfg(test)]
    pub fn computed(num: f32) -> NumberValue {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: true,
        }
    }

    pub fn with_units(num: f32, unit: String) -> NumberValue {
        NumberValue {
            scalar:   num,
            unit:     Some(unit),
            computed: false,
        }
    }

    pub fn apply_math(self, op: Op, nv: NumberValue) -> Result<NumberValue> {
        let result       = try!(self.compute_number(op, &nv));
        let result_units = try!(self.compute_units(op, nv));

        Ok(NumberValue {
            scalar:   result,
            unit:     result_units,
            computed: true,
        })
    }

    fn compute_number(&self, op: Op, nv: &NumberValue) -> Result<f32> {
        op.math(self.scalar, nv.scalar)
    }

    fn compute_units(self, op: Op, nv: NumberValue) -> Result<Option<String>> {
        let unit = match (self.unit, nv.unit) {
            (Some(u), None) | (None, Some(u)) => Some(u),
            (Some(ref u1), Some(ref u2)) if u1 == u2 => {
                match op {
                    Op::Slash => None, // Divide out the units
                    Op::Star => return Err(SassError {
                        kind: ErrorKind::InvalidSquareUnits,
                        message: format!(
                            "Multiplication of {:?} and {:?} would produce invalid squared units",
                            u1, u2
                        ),
                    }),
                    _ => Some(u1.clone()),
                }
            },
            (None, None) => None,
            (other1, other2) => {
                return Err(SassError {
                    kind: ErrorKind::IncompatibleUnits,
                    message: format!(
                        "Incompatible units: {:?} and {:?}",
                        other1, other2
                    ),
                });
            },
        };
        Ok(unit)
    }
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.scalar, self.unit.clone().unwrap_or(String::from("")))
    }
}
