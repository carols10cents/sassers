use sass::output_style::SassOutputStyle;
use token::Lexeme;
use error::Result;

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct SassComment {
    pub content: Lexeme,
}

impl SassComment {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        let comment = self.content.token.to_string();
        let s = match style {
            SassOutputStyle::Nested |
            SassOutputStyle::Expanded => format!("{}\n", comment),
            SassOutputStyle::Compressed => String::from(""),
            SassOutputStyle::Compact => {
                let c = comment.lines().map(|s| s.trim()).collect::<Vec<_>>().join(" ");
                format!("{}\n", c)
            },
            SassOutputStyle::Debug => format!("{:?}\n", self),
            _ => unreachable!(),
        };
        Ok(try!(write!(output, "{}", s)))

    }
}
