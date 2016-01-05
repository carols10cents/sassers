use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::value_part::ValuePart;
use sass::mixin::{SassMixinCall, SassMixin};
use sass::output_style::SassOutputStyle;
use error::{SassError, ErrorKind, Result};

use std::io::Write;

#[derive(Debug, Clone)]
pub enum Event {
    Property(String, ValuePart),
    UnevaluatedProperty(String, String),
    Comment(String),
    Rule(SassRule),
    Variable(SassVariable),
    Mixin(SassMixin),
    MixinCall(SassMixinCall),
    List(Vec<Event>),
}

impl Event {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Event::Rule(ref rule) => {
                Ok(try!(rule.stream(output, style)))
            },
            Event::Comment(ref comment) => {
                let s = match style {
                    SassOutputStyle::Nested |
                    SassOutputStyle::Expanded => format!("{}\n", comment),
                    SassOutputStyle::Compressed => String::from(""),
                    SassOutputStyle::Compact => {
                        let c = comment.lines().map(|s| s.trim()).collect::<Vec<_>>().join(" ");
                        format!("{}\n", c)
                    },
                    SassOutputStyle::Debug => format!("{:?}\n", self),
                };
                Ok(try!(write!(output, "{}", s)))
            },
            Event::List(ref events) => {
                for event in events {
                    try!(event.stream(output, style))
                }
                Ok(())
            },
            ref other => return Err(SassError {
                kind: ErrorKind::UnexpectedTopLevelElement,
                message: format!(
                    "Expceted one of Rule, Comment, or List at the top level of the file; got: `{:?}`",
                    other
                ),
            }),
        }
    }

    pub fn to_string(&self, style: SassOutputStyle) -> String {
        match style {
            SassOutputStyle::Nested => self.nested(),
            SassOutputStyle::Compressed => self.compressed(),
            SassOutputStyle::Expanded => self.expanded(),
            SassOutputStyle::Compact => self.compact(),
            SassOutputStyle::Debug => format!("{:?}\n", self),
        }
    }

    pub fn expanded(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.expanded()),
            Event::Comment(ref comment) => format!("  {}", comment),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn nested(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.nested()),
            Event::Comment(ref comment) => format!("  {}", comment),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn compact(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("{}: {};", name, value.compact()),
            Event::Comment(ref comment) => (*comment).to_string(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn compressed(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => {
                format!("{}:{}", name, value.compressed())
            },
            Event::Comment(..) => String::from(""),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn is_child_rule(&self) -> bool {
        match self {
            &Event::Rule(..) => true,
            _ => false,
        }
    }
}
