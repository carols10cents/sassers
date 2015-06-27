use event::Event;

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Event<'a>,
}

impl <'a> SassComment<'a> {
    pub fn expanded(&self) -> String {
        match &self.comment {
            &Event::Comment(ref c) => (*c).to_string(),
            _ => unreachable!(),
        }
    }

    pub fn nested(&self) -> String {
        match &self.comment {
            &Event::Comment(ref c) => (*c).to_string(),
            _ => unreachable!(),
        }
    }

    pub fn compact(&self) -> String {
        match &self.comment {
            &Event::Comment(ref c) => (*c).to_string().lines().map(|s| s.trim()).collect::<Vec<_>>().connect(" "),
            _ => unreachable!(),
        }
    }

    pub fn compressed(&self) -> String {
        match &self.comment {
            &Event::Comment(ref c) => (*c).to_string(),
            _ => unreachable!(),
        }
    }
}
