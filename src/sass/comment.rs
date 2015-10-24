use std::borrow::Cow;

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Cow<'a, str>,
}

impl <'a> SassComment<'a> {
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
