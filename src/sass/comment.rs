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
}
