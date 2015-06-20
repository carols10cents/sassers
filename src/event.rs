use std::borrow::Cow;
use std::fmt;

#[derive(Clone)]
pub struct SassRule<'a> {
    pub selectors: Vec<SassSelector<'a>>,
    pub selectors_done: bool,
    pub children: Vec<Event<'a>>,
}

impl<'a> SassRule<'a> {
    pub fn new() -> SassRule<'a> {
        SassRule {
            selectors: Vec::new(),
            selectors_done: false,
            children: Vec::new(),
        }
    }

    pub fn expanded(&self) -> String {
        let mut exp = String::new();

        let selector_string = self.selectors.iter().map(|s| (*s.name).to_string()).collect::<Vec<_>>().connect(", ");
        exp.push_str(&selector_string);
        exp.push_str(" ");

        let children_string = self.children.iter().map(|c| c.expanded()).collect::<Vec<_>>().connect("\n  ");

        if self.has_properties() {
            exp.push_str(&format!("{{\n{}}}", children_string));
        } else {
            exp.push_str(&children_string);
        }

        exp
    }

    fn has_properties(&self) -> bool {
        self.children.iter().any(|c| c.is_property() )
    }
}

impl<'a> fmt::Debug for SassRule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children = self.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().connect("\n");
        let indented_children = children.split("\n").collect::<Vec<_>>().connect("\n  ");
        write!(f, "{:?} {{\n  {}\n}}", self.selectors, indented_children)
    }
}

#[derive(Debug,Clone)]
pub struct SassVariable<'a> {
    pub variable: Event<'a>,
}

#[derive(Debug,Clone)]
pub struct SassComment<'a> {
    pub comment: Event<'a>,
}

#[derive(Debug,Clone)]
pub struct SassSelector<'a> {
    pub name: Cow<'a, str>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    OutsideRules,
    InVariable,
    InComment,
    InRule,
    InSelectors,
    InProperties,
    EndRule,
    Eof,
}

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Property(Cow<'a, str>, Cow<'a, str>),
    Variable(Cow<'a, str>, Cow<'a, str>),
    Comment(Cow<'a, str>),
    ChildRule(SassRule<'a>),
    Selector(SassSelector<'a>),
}

impl<'a> Event<'a> {
    pub fn expanded(&self) -> String {
        match (*self).clone() {
            Event::Property(name, value) => format!("  {}: {};\n", name, value),
            Event::Variable(..) => String::new(),
            Event::Comment(comment) => (*comment).to_string(),
            Event::ChildRule(sass_rule) => sass_rule.expanded(),
            Event::Selector(..) => unreachable!(),
        }
    }

    pub fn is_property(&self) -> bool {
        match self {
            &Event::Property(..) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TopLevelEvent<'a> {
    Rule(SassRule<'a>),
    Variable(SassVariable<'a>),
    Comment(SassComment<'a>),
}
