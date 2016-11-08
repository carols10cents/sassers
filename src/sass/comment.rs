use sass::output_style::SassOutputStyle;
use error::Result;
use token_offset::TokenOffset;

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct SassComment {
    pub content: TokenOffset,
}

impl SassComment {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        let comment = self.content.token.to_string();
        let s = match style {
            SassOutputStyle::Nested |
            SassOutputStyle::Expanded => format!("{}", comment),
            SassOutputStyle::Compressed => String::from(""),
            SassOutputStyle::Compact => {
                let c = comment.lines().map(|s| s.trim()).collect::<Vec<_>>().join(" ");
                format!("{}", c)
            },
            SassOutputStyle::Debug => format!("{:#?}\n", self),
            _ => unreachable!(),
        };
        Ok(try!(write!(output, "{}", s)))

    }
}
