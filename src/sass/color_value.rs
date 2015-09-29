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
    pub computed: bool,
    pub original: Cow<'a, str>,
}

impl<'a, 'b> ColorValue<'a> {
    pub fn from_hex(hex: Cow<'a, str>) -> Result<ColorValue<'a>> {
        if hex.len() == 4 {
            Ok(ColorValue {
                red:   u8::from_str_radix(&hex[1..2], 16).unwrap() * 17,
                green: u8::from_str_radix(&hex[2..3], 16).unwrap() * 17,
                blue:  u8::from_str_radix(&hex[3..4], 16).unwrap() * 17,
                computed: false,
                original: hex,
            })
        } else if hex.len() == 7 {
            Ok(ColorValue {
                red:   u8::from_str_radix(&hex[1..3], 16).unwrap(),
                green: u8::from_str_radix(&hex[3..5], 16).unwrap(),
                blue:  u8::from_str_radix(&hex[5..7], 16).unwrap(),
                computed: false,
                original: hex,
            })
        } else {
            Err(SassError {
                kind: ErrorKind::InvalidColor,
                message: format!("Invalid hex color: {}", hex),
            })
        }
    }

    pub fn from_computed(r: u8, g: u8, b: u8) -> ColorValue<'a> {
        ColorValue {
            red: r, green: g, blue: b,
            computed: true,
            original: hex_format(r, g, b).into(),
        }
    }

    pub fn into_owned(self) -> ColorValue<'b> {
        ColorValue {
            red: self.red, green: self.green, blue: self.blue,
            computed: self.computed,
            original: self.original.into_owned().into(),
        }
    }

    pub fn apply_math(self, op: Op, nv: NumberValue<'a>) -> Result<ColorValue<'a>> {
        let num   = nv.scalar as u8;
        let red   = try!(saturating_math(op, self.red, num));
        let green = try!(saturating_math(op, self.green, num));
        let blue  = try!(saturating_math(op, self.blue, num));
        Ok(ColorValue::from_computed(red, green, blue))
    }

    pub fn combine_colors(self, op: Op, c: ColorValue<'a>) -> Result<ColorValue<'a>> {
        let red   = try!(saturating_math(op, self.red, c.red));
        let green = try!(saturating_math(op, self.green, c.green));
        let blue  = try!(saturating_math(op, self.blue, c.blue));
        Ok(ColorValue::from_computed(red, green, blue))
    }

    pub fn to_short_hex(&self) -> String {
        if self.red % 17 == 0 && self.green % 17 == 0 && self.blue % 17 == 0 {
            format!("#{:x}{:x}{:x}", self.red / 17, self.green / 17, self.blue / 17)
        } else {
            hex_format(self.red, self.green, self.blue)
        }
    }

    pub fn compressed(&self) -> String {
        if self.computed {
            if self.to_short_hex().len() < self.to_named_color().len() {
                self.to_short_hex()
            } else {
                self.to_named_color()
            }
        } else {
            format!("{}", self)
        }
    }

    pub fn to_named_color(&self) -> String {
        // TODO once we have alpha channel: 'transparent', 0x00000000

        String::from(match (self.red, self.green, self.blue) {
            // standard colors
            (0, 255, 255)   => "cyan",    // Sass prefers cyan over aqua
            (0, 0, 0)       => "black",
            (0, 0, 255)     => "blue",
            (255, 0, 255)   => "magenta", // Sass prefers magenta over fuchsia
            (80, 80, 80)    => "gray",    // Sass prefers this spelling over grey
            (0, 80, 0)      => "green",
            (0, 255, 0)     => "lime",
            (80, 0, 0)      => "maroon",
            (0, 0, 80)      => "navy",
            (80, 80, 0)     => "olive",
            (80, 0, 80)     => "purple",
            (255, 0, 0)     => "red",
            (192, 192, 192) => "silver",
            (0, 80, 80)     => "teal",
            (255, 255, 255) => "white",
            (255, 255, 0)   => "yellow",
            // the rest of the named colors, converted as I get around to it
            (169, 169, 169) => "darkgrey",
            (47, 79, 79)    => "darkslategrey",
            (69, 69, 69)    => "dimgrey",
            (211, 211, 211) => "lightgrey",
            (77, 88, 99)    => "lightslategrey",
            (70, 80, 90)    => "slategrey",
            (240, 248, 255) => "aliceblue",
            (250, 235, 215) => "antiquewhite",
            (127, 255, 212) => "aquamarine",
            (240, 255, 255) => "azure",
            (245, 245, 220) => "beige",
            (255, 228, 196) => "bisque",
            (255, 235, 205) => "blanchedalmond",
            (138, 43, 226)  => "blueviolet",
            (165, 42, 42)   => "brown",
            (222, 184, 135) => "burlywood",
            (95, 158, 160)  => "cadetblue",

            (r, g, b)       => return hex_format(r, g, b),
        })
               // 'chartreuse'           => 7FFF00,
               // 'chocolate'            => D2691E,
               // 'coral'                => FF7F50,
               // 'cornflowerblue'       => 6495ED,
               // 'cornsilk'             => FFF8DC,
               // 'crimson'              => DC143C,
               // 'darkblue'             => 00008B,
               // 'darkcyan'             => 008B8B,
               // 'darkgoldenrod'        => B8860B,
               // 'darkgray'             => A9A9A9,
               // 'darkgreen'            => 006400,
               // 'darkkhaki'            => BDB76B,
               // 'darkmagenta'          => 8B008B,
               // 'darkolivegreen'       => 556B2F,
               // 'darkorange'           => FF8C00,
               // 'darkorchid'           => 9932CC,
               // 'darkred'              => 8B0000,
               // 'darksalmon'           => E9967A,
               // 'darkseagreen'         => 8FBC8F,
               // 'darkslateblue'        => 483D8B,
               // 'darkslategray'        => 2F4F4F,
               // 'darkturquoise'        => 00CED1,
               // 'darkviolet'           => 9400D3,
               // 'deeppink'             => FF1493,
               // 'deepskyblue'          => 00BFFF,
               // 'dimgray'              => 696969,
               // 'dodgerblue'           => 1E90FF,
               // 'firebrick'            => B22222,
               // 'floralwhite'          => FFFAF0,
               // 'forestgreen'          => 228B22,
               // 'gainsboro'            => DCDCDC,
               // 'ghostwhite'           => F8F8FF,
               // 'gold'                 => FFD700,
               // 'goldenrod'            => DAA520,
               // 'greenyellow'          => ADFF2F,
               // 'honeydew'             => F0FFF0,
               // 'hotpink'              => FF69B4,
               // 'indianred'            => CD5C5C,
               // 'indigo'               => 4B0082,
               // 'ivory'                => FFFFF0,
               // 'khaki'                => F0E68C,
               // 'lavender'             => E6E6FA,
               // 'lavenderblush'        => FFF0F5,
               // 'lawngreen'            => 7CFC00,
               // 'lemonchiffon'         => FFFACD,
               // 'lightblue'            => ADD8E6,
               // 'lightcoral'           => F08080,
               // 'lightcyan'            => E0FFFF,
               // 'lightgoldenrodyellow' => FAFAD2,
               // 'lightgreen'           => 90EE90,
               // 'lightgray'            => D3D3D3,
               // 'lightpink'            => FFB6C1,
               // 'lightsalmon'          => FFA07A,
               // 'lightseagreen'        => 20B2AA,
               // 'lightskyblue'         => 87CEFA,
               // 'lightslategray'       => 778899,
               // 'lightsteelblue'       => B0C4DE,
               // 'lightyellow'          => FFFFE0,
               // 'limegreen'            => 32CD32,
               // 'linen'                => FAF0E6,
               // 'mediumaquamarine'     => 66CDAA,
               // 'mediumblue'           => 0000CD,
               // 'mediumorchid'         => BA55D3,
               // 'mediumpurple'         => 9370DB,
               // 'mediumseagreen'       => 3CB371,
               // 'mediumslateblue'      => 7B68EE,
               // 'mediumspringgreen'    => 00FA9A,
               // 'mediumturquoise'      => 48D1CC,
               // 'mediumvioletred'      => C71585,
               // 'midnightblue'         => 191970,
               // 'mintcream'            => F5FFFA,
               // 'mistyrose'            => FFE4E1,
               // 'moccasin'             => FFE4B5,
               // 'navajowhite'          => FFDEAD,
               // 'oldlace'              => FDF5E6,
               // 'olivedrab'            => 6B8E23,
               // 'orange'               => FFA500,
               // 'orangered'            => FF4500,
               // 'orchid'               => DA70D6,
               // 'palegoldenrod'        => EEE8AA,
               // 'palegreen'            => 98FB98,
               // 'paleturquoise'        => AFEEEE,
               // 'palevioletred'        => DB7093,
               // 'papayawhip'           => FFEFD5,
               // 'peachpuff'            => FFDAB9,
               // 'peru'                 => CD853F,
               // 'pink'                 => FFC0CB,
               // 'plum'                 => DDA0DD,
               // 'powderblue'           => B0E0E6,
               // 'rebeccapurple'        => 663399,
               // 'rosybrown'            => BC8F8F,
               // 'royalblue'            => 4169E1,
               // 'saddlebrown'          => 8B4513,
               // 'salmon'               => FA8072,
               // 'sandybrown'           => F4A460,
               // 'seagreen'             => 2E8B57,
               // 'seashell'             => FFF5EE,
               // 'sienna'               => A0522D,
               // 'skyblue'              => 87CEEB,
               // 'slateblue'            => 6A5ACD,
               // 'slategray'            => 708090,
               // 'snow'                 => FFFAFA,
               // 'springgreen'          => 00FF7F,
               // 'steelblue'            => 4682B4,
               // 'tan'                  => D2B48C,
               // 'thistle'              => D8BFD8,
               // 'tomato'               => FF6347,
               // 'turquoise'            => 40E0D0,
               // 'violet'               => EE82EE,
               // 'wheat'                => F5DEB3,
               // 'whitesmoke'           => F5F5F5,
               // 'yellowgreen'          => 9ACD32,
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

fn hex_format(red: u8, green: u8, blue: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", red, green, blue)
}

impl<'a> fmt::Display for ColorValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let candidate = if self.computed {
            self.to_named_color()
        } else {
            hex_format(self.red, self.green, self.blue)
        };
        if candidate.len() <= self.original.len() {
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
        assert!(res.computed);
        assert_eq!("#ff0101", format!("{}", res));
    }

    #[test]
    fn it_prefers_named_colors_if_computed() {
        let c = ColorValue::from_computed(192, 192, 192);
        assert_eq!("silver", format!("{}", c));
    }

    #[test]
    fn combining_colors_results_in_computed() {
        let c = ColorValue::from_hex(Borrowed("#ff0000")).unwrap();
        let d = ColorValue::from_hex(Borrowed("#00ff00")).unwrap();
        let res = c.combine_colors(Op::Plus, d).unwrap();
        assert!(res.computed);
        assert_eq!("yellow", format!("{}", res));
    }
}
