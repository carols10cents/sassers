use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorValue {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
}

impl<'a> ColorValue {
    pub fn from_hex(hex: Cow<'a, str>) -> ColorValue {
        if hex.len() == 4 {
            ColorValue {
                red:   i32::from_str_radix(&hex[1..2], 16).unwrap() * 17,
                green: i32::from_str_radix(&hex[2..3], 16).unwrap() * 17,
                blue:  i32::from_str_radix(&hex[3..4], 16).unwrap() * 17,
            }
        } else if hex.len() == 7 {
            ColorValue {
                red:   i32::from_str_radix(&hex[1..3], 16).unwrap(),
                green: i32::from_str_radix(&hex[3..5], 16).unwrap(),
                blue:  i32::from_str_radix(&hex[5..7], 16).unwrap(),
            }
        } else {
            panic!("Invalid hex color: {}", hex); // TODO: Result
        }
    }
}

impl fmt::Display for ColorValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}{}{}", self.red, self.green, self.blue)
    }
}
