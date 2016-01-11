use std::hash;
use std::fmt;

#[derive(Clone, Debug, Eq)]
pub struct Token {
    pub value: String,
    pub offset: Option<usize>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool { PartialEq::eq(&self.value, &other.value) }
}

impl hash::Hash for Token {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        (*self.value).hash(hasher)
    }
}

impl<'a> Token {
    pub fn new(s: &'a str, o: usize) -> Token {
        Token {
            value: String::from(s),
            offset: Some(o),
        }
    }
}

impl<'a> From<&'a str> for Token {
    fn from(s: &'a str) -> Token {
        Token {
            value: String::from(s),
            offset: None,
        }
    }
}
