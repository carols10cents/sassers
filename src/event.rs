use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::value_part::ValuePart;
use sass::mixin::{SassMixinCall, SassMixin};
use sass::output_style::SassOutputStyle;
use error::{SassError, ErrorKind, Result};

use std::borrow::Cow;
use std::io::Write;

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Property(Cow<'a, str>, ValuePart<'a>),
    UnevaluatedProperty(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
    Rule(SassRule<'a>),
    Variable(SassVariable<'a>),
    Mixin(SassMixin<'a>),
    MixinCall(SassMixinCall<'a>),
    List(Vec<Event<'a>>),
}

impl<'a> Event<'a> {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Event::Rule(ref rule) => {
                write!(output, "{}", rule.output(style)).map_err(|e| {
                    SassError::from(e)
                })
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
                write!(output, "{}", s).map_err(|e| {
                    SassError::from(e)
                })
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

    pub fn expanded(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.expanded()),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::Rule(ref sass_rule) => sass_rule.expanded(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn nested(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("  {}: {};", name, value.nested()),
            Event::Comment(ref comment) => format!("  {}", comment),
            Event::Rule(ref sass_rule) => sass_rule.nested(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn compact(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => format!("{}: {};", name, value.compact()),
            Event::Comment(ref comment) => (*comment).to_string(),
            Event::Rule(ref sass_rule) => sass_rule.compact(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn compressed(&self) -> String {
        match *self {
            Event::Property(ref name, ref value) => {
                format!("{}:{}", name, value.compressed())
            },
            Event::Comment(..) => String::from(""),
            Event::Rule(ref sass_rule) => sass_rule.compressed(),
            ref other => format!("other = {:?}", other),
        }
    }

    pub fn is_child_rule(&self) -> bool {
        match self {
            &Event::Rule(..) => true,
            _ => false,
        }
    }

    pub fn is_comment(&self) -> bool {
        match self {
            &Event::Comment(..) => true,
            _ => false,
        }
    }
}
