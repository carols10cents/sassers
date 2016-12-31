use sass::output_style::{SassOutputStyle, Streamable};
use error::Result;
use token_offset::TokenOffset;

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct SassComment {
    pub content: TokenOffset,
}

impl Streamable for SassComment {
    fn stream(&self, output: &mut Write, style: Box<SassOutputStyle>)
                        -> Result<()> {
        let comment = self.content.token.to_string();
        // TODO: Shouldn't write! call into here, and not need the call to try?
        try!(write!(output, "{}", style.comment(&comment)));
        Ok(())
    }
}
