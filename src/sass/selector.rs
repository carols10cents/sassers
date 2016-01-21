use regex::Regex;
use regex::Captures;
use token::Lexeme;

#[derive(Debug,Clone)]
pub struct SassSelector {
    pub name: Lexeme,
}

fn squeeze(data: String) -> String {
    let re = Regex::new(r"\s{2,}").unwrap();
    re.replace_all(&data[..], " ")
}

fn compress_attr_selectors(data: String) -> String {
    let re = Regex::new(r"\[\s*(?P<attrname>[^\s*~^|=]+)\s*(?P<operator>[*~^$|]?=)\s*(?P<attrval>[^\s\]]+)\s*\]").unwrap();
    re.replace(&data[..], |caps: &Captures| {
        format!("[{}{}{}]", caps.at(1).unwrap_or(""), caps.at(2).unwrap_or(""), caps.at(3).unwrap_or(""))
    })
}

impl SassSelector {
    pub fn new(selector: Lexeme) -> SassSelector {
        SassSelector {
            name: Lexeme {
                token: compress_attr_selectors(squeeze(selector.value())),
                offset: selector.offset,
            }
        }
    }
}
