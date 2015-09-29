use error::{SassError, ErrorKind, Result};
use sass::op::Op;
use sass::number_value::NumberValue;

use std::borrow::Cow;
use std::fmt;
use std::cmp;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorValue<'a> {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub original: Cow<'a, str>,
}

impl<'a, 'b> ColorValue<'a> {
    pub fn from_hex(hex: Cow<'a, str>) -> Result<ColorValue<'a>> {
        if hex.len() == 4 {
            Ok(ColorValue {
                red:   u8::from_str_radix(&hex[1..2], 16).unwrap() * 17,
                green: u8::from_str_radix(&hex[2..3], 16).unwrap() * 17,
                blue:  u8::from_str_radix(&hex[3..4], 16).unwrap() * 17,
                original: hex,
            })
        } else if hex.len() == 7 {
            Ok(ColorValue {
                red:   u8::from_str_radix(&hex[1..3], 16).unwrap(),
                green: u8::from_str_radix(&hex[3..5], 16).unwrap(),
                blue:  u8::from_str_radix(&hex[5..7], 16).unwrap(),
                original: hex,
            })
        } else {
            Err(SassError {
                kind: ErrorKind::InvalidColor,
                message: format!("Invalid hex color: {}", hex),
            })
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> ColorValue<'a> {
        ColorValue {
            red: r, green: g, blue: b, original: format!("#{:02x}{:02x}{:02x}", r, g, b).into(),
        }
    }

    pub fn into_owned(self) -> ColorValue<'b> {
        ColorValue {
            red: self.red, green: self.green, blue: self.blue,
            original: self.original.into_owned().into(),
        }
    }

    pub fn apply_math(self, op: Op, nv: NumberValue<'a>) -> Result<ColorValue<'a>> {
        let num = nv.scalar as u8;
        Ok(ColorValue::from_rgb(
            try!(saturating_math(op, self.red, num)),
            try!(saturating_math(op, self.green, num)),
            try!(saturating_math(op, self.blue, num)),
        ))
    }

    pub fn combine_colors(self, op: Op, c: ColorValue<'a>) -> Result<ColorValue<'a>> {
        Ok(ColorValue::from_rgb(
            try!(saturating_math(op, self.red, c.red)),
            try!(saturating_math(op, self.green, c.green)),
            try!(saturating_math(op, self.blue, c.blue)),
        ))
    }
}

// Not the color kind of saturating.
fn saturating_math(op: Op, a: u8, b: u8) -> Result<u8> {
    Ok(match op {
        Op::Plus    => a.saturating_add(b),
        Op::Minus   => a.saturating_sub(b),
        Op::Star    => cmp::max(cmp::min(a as i32 * b as i32, 255), 0) as u8,
        Op::Slash   => cmp::max(cmp::min(a as i32 / b as i32, 255), 0) as u8,
        Op::Percent => cmp::max(cmp::min(a as i32 % b as i32, 255), 0) as u8,
        other => return Err(SassError {
            kind: ErrorKind::InvalidOperator,
            message: format!(
                "Cannot apply operator {:?} on color as math",
                other
            ),
        }),

    })
}

impl<'a> fmt::Display for ColorValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let candidate = format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue);
        if candidate.len() < self.original.len() {
            write!(f, "{}", candidate)
        } else {
            write!(f, "{}", self.original)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::number_value::NumberValue;
    use sass::op::Op;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_ignores_overflow_when_not_a_named_color() {
        let c = ColorValue::from_hex(Borrowed("#ff0000")).unwrap();
        let res = c.apply_math(Op::Plus, NumberValue::from_scalar(1.0)).unwrap();
        assert_eq!("#ff0101", format!("{}", res));
    }
}
