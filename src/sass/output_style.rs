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
