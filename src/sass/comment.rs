use std::borrow::Cow;

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Cow<'a, str>,
}

impl <'a> SassComment<'a> {
    pub fn expanded(&self) -> String {
      self.comment.to_string()
    }

    pub fn nested(&self) -> String {
      self.comment.to_string()
    }

    pub fn compact(&self) -> String {
      self.comment.to_string().lines().map(|s| s.trim()).collect::<Vec<_>>().join(" ")
    }

    // Comments are never output in the compressed format.
    #[allow(dead_code)]
    pub fn compressed(&self) -> String {
        unreachable!()
    }
}
