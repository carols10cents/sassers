use sass::output_style::SassOutputStyle;
use sass::rule::SassRule;
use error::{Result};

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Root {
    Rule(SassRule),
}

impl Root {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Root::Rule(ref sr) => sr.stream(output, style),
        }
    }
}
