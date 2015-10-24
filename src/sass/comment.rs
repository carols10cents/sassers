use sass::output_style::SassOutputStyle;

use std::borrow::Cow;

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Cow<'a, str>,
}

impl <'a> SassComment<'a> {
    pub fn output(&self, style: SassOutputStyle) -> String {
        match style {
            SassOutputStyle::Nested => self.nested(),
            SassOutputStyle::Compressed => self.compressed(),
            SassOutputStyle::Expanded => self.expanded(),
            SassOutputStyle::Compact => self.compact(),
            SassOutputStyle::Debug => format!("{:?}\n", self),
        }
    }

    pub fn expanded(&self) -> String {
        format!("{}\n", self.comment)
    }

    pub fn nested(&self) -> String {
        format!("{}\n", self.comment)
    }

    pub fn compact(&self) -> String {
      format!("{}\n", self.comment.to_string().lines().map(|s| s.trim()).collect::<Vec<_>>().join(" "))
    }

    pub fn compressed(&self) -> String {
        String::from("")
    }
}
