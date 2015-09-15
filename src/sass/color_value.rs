use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorValue<'a> {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
    pub original: Cow<'a, str>,
}

impl<'a, 'b> ColorValue<'a> {
    pub fn from_hex(hex: Cow<'a, str>) -> ColorValue<'a> {
        if hex.len() == 4 {
            ColorValue {
                red:   i32::from_str_radix(&hex[1..2], 16).unwrap() * 17,
                green: i32::from_str_radix(&hex[2..3], 16).unwrap() * 17,
                blue:  i32::from_str_radix(&hex[3..4], 16).unwrap() * 17,
                original: hex,
            }
        } else if hex.len() == 7 {
            ColorValue {
                red:   i32::from_str_radix(&hex[1..3], 16).unwrap(),
                green: i32::from_str_radix(&hex[3..5], 16).unwrap(),
                blue:  i32::from_str_radix(&hex[5..7], 16).unwrap(),
                original: hex,
            }
        } else {
            panic!("Invalid hex color: {}", hex); // TODO: Result
        }
    }

    pub fn into_owned(self) -> ColorValue<'b> {
        ColorValue {
            red: self.red, green: self.green, blue: self.blue,
            original: self.original.into_owned().into(),
        }
    }
}

impl<'a> fmt::Display for ColorValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let candidate = format!("#{}{}{}", self.red, self.green, self.blue);
        if candidate.len() < self.original.len() {
            write!(f, "{}", candidate)
        } else {
            write!(f, "{}", self.original)
        }
    }
}
