use std::borrow::Cow;
use regex::Regex;
use regex::Captures;

#[derive(Debug,Clone)]
pub struct SassSelector<'a> {
    pub name: Cow<'a, str>,
}

fn squeeze(data: &str) -> String {
    let re = Regex::new(r"\s{2,}").unwrap();
    re.replace_all(data, " ")
}

fn compress_attr_selectors(data: &str) -> String {
    let re = Regex::new(r"\[\s*(?P<attrname>[^\s*~^|=]+)\s*(?P<operator>[*~^$|]?=)\s*(?P<attrval>[^\s\]]+)\s*\]").unwrap();
    re.replace(data, |caps: &Captures| {
        format!("[{}{}{}]", caps.at(1).unwrap_or(""), caps.at(2).unwrap_or(""), caps.at(3).unwrap_or(""))
    })
}

impl<'a> SassSelector<'a> {
    pub fn new(selector_str: &'a str) -> SassSelector<'a> {
        SassSelector { name: compress_attr_selectors(&squeeze(selector_str)).into() }
    }
}
