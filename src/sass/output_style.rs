use error::{Result, SassError, ErrorKind};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SassOutputStyle {
    Expanded,
    Nested,
    Compact,
    Compressed,
    Debug,
}

impl FromStr for SassOutputStyle {
    type Err = SassError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "nested"     => Ok(SassOutputStyle::Nested),
            "compressed" => Ok(SassOutputStyle::Compressed),
            "expanded"   => Ok(SassOutputStyle::Expanded),
            "compact"    => Ok(SassOutputStyle::Compact),
            "debug"      => Ok(SassOutputStyle::Debug),
            style        => Err(SassError {
                kind: ErrorKind::InvalidOutputStyle,
                message: format!("Unknown output style {:?}. Please specify one of nested, compressed, expanded, or compact.", style),
            }),
        }
    }
}

impl SassOutputStyle {
    pub fn rule_separator(&self) -> String {
        String::from(match *self {
            SassOutputStyle::Compressed => "",
            _ => "\n\n",
        })
    }

    pub fn selector_separator(&self) -> String {
        String::from(match *self {
            SassOutputStyle::Compressed => ",",
            _ => ", ",
        })
    }

    pub fn selector_brace_separator(&self) -> String {
        String::from(match *self {
            SassOutputStyle::Compressed => "",
            _ => " ",
        })
    }

    pub fn brace_property_separator(&self) -> String {
        String::from(match *self {
            SassOutputStyle::Nested => "\n",
            SassOutputStyle::Expanded => "\n",
            SassOutputStyle::Compact => " ",
            _ => "",
        })
    }

    pub fn property_separator(&self, parents: &str) -> String {
        match *self {
            SassOutputStyle::Compact => String::from(" "),
            SassOutputStyle::Compressed => String::from(";"),
            SassOutputStyle::Nested => format!("\n{}", parents),
            _ => String::from("\n"),
        }
    }

    pub fn property_brace_separator(&self) -> String {
        String::from(match *self {
            SassOutputStyle::Nested => " ",
            SassOutputStyle::Expanded => "\n",
            SassOutputStyle::Compact => " ",
            _ => "",
        })
    }

    pub fn rule_and_child_rules_separator(&self) -> String {
        String::from(match *self {
            SassOutputStyle::Nested => "\n  ",
            SassOutputStyle::Expanded => "\n",
            SassOutputStyle::Compact => "\n",
            _ => "",
        })
    }

    pub fn child_rule_separator(&self, has_properties: bool) -> String {
        String::from(match (*self, has_properties) {
            (SassOutputStyle::Nested, true) => "\n  ",
            (SassOutputStyle::Compressed, _) => "",
            (_, _) => "\n",
        })
    }
}