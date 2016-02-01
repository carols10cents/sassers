use error::{Result, SassError, ErrorKind};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SassOutputStyle {
    Expanded,
    Nested,
    Compact,
    Compressed,
    Debug,
    Tokens,
    AST,
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
            "tokens"     => Ok(SassOutputStyle::Tokens),
            "ast"        => Ok(SassOutputStyle::AST),
            style        => Err(SassError {
                offset: 0,
                kind: ErrorKind::InvalidOutputStyle,
                // Intentionally hiding debug/tokens/ast
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

    pub fn before_property(&self, nesting: &str) -> String {
        match *self {
            SassOutputStyle::Compact => String::from(""),
            SassOutputStyle::Compressed => String::from(""),
            SassOutputStyle::Nested => String::from(nesting),
            _ => String::from("\n"),
        }
    }

    pub fn after_property(&self) -> String {
        match *self {
            SassOutputStyle::Compact => String::from(" "),
            SassOutputStyle::Compressed => String::from(";"),
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
}