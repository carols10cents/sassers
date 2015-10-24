use event::Event;
use sass::selector::SassSelector;

use std::fmt;

#[derive(Clone)]
pub struct SassRule<'a> {
    pub selectors: Vec<SassSelector<'a>>,
    pub children: Vec<Event<'a>>,
}

impl<'a> SassRule<'a> {
    pub fn new() -> SassRule<'a> {
        SassRule {
            selectors: Vec::new(),
            children: Vec::new(),
        }
    }

    fn selector_distribution(&self, parents: &str, separator: &str) -> String {
        match parents.len() {
            0 => self.selectors.iter().map(|s| (*s.name).to_string()).collect::<Vec<_>>().join(separator),
            _ => parents.split(",").map(|p| {
                self.selectors.iter().map(|s| {
                    if s.name.contains("&") {
                        s.name.replace("&", p.trim())
                    } else {
                        format!("{} {}", p.trim(), s.name)
                    }
                }).collect::<Vec<_>>().join(separator)
            }).collect::<Vec<_>>().join(separator),
        }
    }

    pub fn expanded(&self) -> String {
        format!("{}\n\n", self.expanded_with_parent(""))
    }

    pub fn expanded_with_parent(&self, parents: &str) -> String {
        let mut output = String::new();

        let selector_string = self.selector_distribution(parents, ", ");

        let properties_string = self.children.iter().filter(|c| !c.is_child_rule() ).map(|c| {
            c.expanded()
        }).collect::<Vec<_>>().join("\n");

        let child_rules_string = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
            match c {
                &Event::ChildRule(ref rule) => rule.expanded_with_parent(&selector_string),
                _ => unreachable!(),
            }
        }).collect::<Vec<_>>().join("\n");

        if properties_string.len() > 0 {
            output.push_str(&selector_string);
            output.push_str(" ");
            output.push_str(&format!("{{\n{}\n}}", properties_string));
        }

        if properties_string.len() > 0 && child_rules_string.len() > 0 {
            output.push_str("\n");
        }

        if child_rules_string.len() > 0 {
            output.push_str(&child_rules_string);
        }

        output
    }

    pub fn nested(&self) -> String {
        format!("{}\n\n", self.nested_with_parent(""))
    }

    pub fn nested_with_parent(&self, parents: &str) -> String {
        let mut output = String::new();

        let selector_string = self.selector_distribution(parents, ", ");

        let properties_string = self.children.iter().filter(|c| !c.is_child_rule() ).map(|c| {
            c.nested()
        }).collect::<Vec<_>>().join("\n");

        let child_rules_string = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
            match c {
                &Event::ChildRule(ref rule) => rule.nested_with_parent(&selector_string),
                _ => unreachable!(),
            }
        }).collect::<Vec<_>>().join("\n");

        if properties_string.len() > 0 {
            output.push_str(&selector_string);
            output.push_str(" ");
            output.push_str(&format!("{{\n{} }}", properties_string));
            if child_rules_string.len() > 0 {
                output.push_str("\n  ");
                output.push_str(&child_rules_string.split('\n').collect::<Vec<_>>().join("\n  "));
            }
        } else {
            output.push_str(&child_rules_string);
        }

        output
    }

    pub fn compact(&self) -> String {
        format!("{}\n\n", self.compact_with_parent(""))
    }

    pub fn compact_with_parent(&self, parents: &str) -> String {
        let mut output = String::new();

        let selector_string = self.selector_distribution(parents, ", ");

        let properties_string = self.children.iter().filter(|c| !c.is_child_rule() ).map(|c| {
            c.compact()
        }).collect::<Vec<_>>().join(" ");

        let child_rules_string = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
            match c {
                &Event::ChildRule(ref rule) => rule.compact_with_parent(&selector_string),
                _ => unreachable!(),
            }
        }).collect::<Vec<_>>().join("\n");

        if properties_string.len() > 0 {
            output.push_str(&selector_string);
            output.push_str(" ");
            output.push_str(&format!("{{ {} }}", properties_string));
            if child_rules_string.len() > 0 {
                output.push_str("\n");
                output.push_str(&child_rules_string);
            }
        } else {
            output.push_str(&child_rules_string);
        }

        output
    }

    pub fn compressed(&self) -> String {
        self.compressed_with_parent("")
    }

    pub fn compressed_with_parent(&self, parents: &str) -> String {
        let mut output = String::new();

        let selector_string = compress_selectors(self.selector_distribution(parents, ","));

        let properties_string = self.children.iter().filter(|c| !c.is_child_rule() && !c.is_comment() ).map(|c| {
            c.compressed()
        }).collect::<Vec<_>>().join(";");

        let child_rules_string = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
            match c {
                &Event::ChildRule(ref rule) => rule.compressed_with_parent(&selector_string),
                _ => unreachable!(),
            }
        }).collect::<Vec<_>>().join("");

        if properties_string.len() > 0 {
            output.push_str(&selector_string);
            output.push_str(&format!("{{{}}}", properties_string));
            if child_rules_string.len() > 0 {
                output.push_str(&child_rules_string);
            }
        } else {
            output.push_str(&child_rules_string);
        }

        output
    }
}

fn compress_selectors(selector_string: String) -> String {
    selector_string.replace(" > ", ">").replace(" + ", "+")
}

impl<'a> fmt::Debug for SassRule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children = self.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join("\n");
        let indented_children = children.split("\n").collect::<Vec<_>>().join("\n  ");
        write!(f, "{:?} {{\n  {}\n}}", self.selectors, indented_children)
    }
}
