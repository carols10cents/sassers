use std::borrow::Cow::Borrowed;
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue<'a> {
    pub scalar:   f32,
    pub unit:     Option<Cow<'a, str>>,
    pub computed: bool,
}

impl<'a> NumberValue<'a> {
    pub fn from_scalar(num: f32) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: false,
        }
    }

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
}

impl<'a> fmt::Display for NumberValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.scalar, self.unit.clone().unwrap_or(Borrowed("")))
    }
}
