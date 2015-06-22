use event::Event;
use sass::selector::SassSelector;

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
        self.expanded_with_parent("")
    }

    pub fn expanded_with_parent(&self, parents: &str) -> String {
        let mut exp = String::new();

        let selector_string = self.selectors.iter().map(|s| {
            match parents.len() {
                0 => (*s.name).to_string(),
                _ => format!("{} {}", parents, s.name),
            }
        }).collect::<Vec<_>>().connect(", ");

        let properties_string = self.children.iter().filter(|c| !c.is_child_rule() ).map(|c| {
            c.expanded()
        }).collect::<Vec<_>>().connect("\n");

        let child_rules_string = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
            match c {
                &Event::ChildRule(ref rule) => rule.expanded_with_parent(&selector_string),
                _ => unreachable!(),
            }
        }).collect::<Vec<_>>().connect("\n");

        if properties_string.len() > 0 {
            exp.push_str(&selector_string);
            exp.push_str(" ");
            exp.push_str(&format!("{{\n{}\n}}\n", properties_string));
            exp.push_str(&child_rules_string);
        } else {
            exp.push_str(&child_rules_string);
        }

        exp
    }

    pub fn map_over_property_values<F>(self, f: &F) -> SassRule<'a>
        where F: Fn(Cow<'a, str>) -> Cow<'a, str>
    {
        let replacement_children = self.children.into_iter().map(|c|
            match c {
                Event::Property(name, value) => {
                    Event::Property(name, f(value))
                },
                Event::ChildRule(rule) => {
                    Event::ChildRule(rule.map_over_property_values(f))
                },
                other => other
            }
        ).collect();

        SassRule {
            children: replacement_children, ..self
        }
    }
}

impl<'a> fmt::Debug for SassRule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children = self.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().connect("\n");
        let indented_children = children.split("\n").collect::<Vec<_>>().connect("\n  ");
        write!(f, "{:?} {{\n  {}\n}}", self.selectors, indented_children)
    }
}
