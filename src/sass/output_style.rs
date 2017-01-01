use ast::node::Node;
use sass::rule::SassRule;
use error::Result;

use std::io::Write;

pub trait Streamable {
    fn stream(&self, output: &mut Write, style: &SassOutputStyle)
             -> Result<()>;
}

pub trait SassOutputStyle {
    fn rule_separator(&self) -> String {
        String::from("\n\n")
    }

    fn selector_separator(&self) -> String {
        String::from(", ")
    }

    fn selector_string(&self, rule: &SassRule, parents: &str) -> String {
        let separator = self.selector_separator();
        if parents.is_empty() {
            rule.selectors.iter()
                          .map(|s| s.token.to_string())
                          .collect::<Vec<_>>().join(&separator)
        } else {
            parents.split(",").map(|p| {
                rule.selectors.iter().map(|s| {
                    if s.token.to_string().contains("&") {
                        s.token.to_string().replace("&", p.trim())
                    } else {
                        format!("{} {}", p.trim(), s.token)
                    }
                }).collect::<Vec<_>>().join(&separator)
            }).collect::<Vec<_>>().join(&separator)
        }
    }

    fn selector_brace_separator(&self) -> String {
        String::from(" ")
    }

    fn brace_property_separator(&self) -> String {
        String::from("")
    }

    fn before_property(&self, _nesting: &str) -> String {
        String::from("\n")
    }

    fn after_property(&self) -> String {
        String::from("")
    }

    fn property_brace_separator(&self) -> String {
        String::from("")
    }

    fn rule_and_child_rules_separator(&self, _nesting: &str) -> String {
        String::from("")
    }

    fn child_rule_separator(&self, _has_properties: bool) -> String {
        String::from("\n")
    }

    fn property(&self, name: &str, value: &str) -> String {
        format!("  {}: {};", name, value)
    }

    fn before_comment(&self) -> String {
        String::new()
    }

    fn comment(&self, content: &str) -> String {
        String::from(content)
    }

    fn after_comment(&self) -> String {
        String::from("\n")
    }

    fn filter_child_properties<'a>(&self, children: &[Node])
       -> Vec<Node> {
        children.iter().filter(|c|
            match **c {
                Node::Rule(..)     => false,
                Node::Comment(..)  => true,
                Node::Property(..) => true,
                Node::Variable(..) => true,
            }
        ).cloned().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Expanded {}

impl SassOutputStyle for Expanded {
    fn brace_property_separator(&self) -> String {
        String::from("")
    }

    fn property_brace_separator(&self) -> String {
        String::from("\n")
    }

    fn rule_and_child_rules_separator(&self, _nesting: &str) -> String {
        String::from("\n")
    }

    fn before_comment(&self) -> String {
        String::from("  ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nested {}

impl SassOutputStyle for Nested {
    fn brace_property_separator(&self) -> String {
        String::from("\n")
    }

    fn before_property(&self, nesting: &str) -> String {
        String::from(nesting)
    }

    fn after_property(&self) -> String {
        String::from("\n")
    }

    fn property_brace_separator(&self) -> String {
        String::from(" ")
    }

    fn rule_and_child_rules_separator(&self, nesting: &str) -> String {
        format!("\n{}", nesting)
    }

    fn child_rule_separator(&self, has_properties: bool) -> String {
        if has_properties {
            String::from("\n  ")
        } else {
            String::from("\n")
        }
    }

    fn before_comment(&self) -> String {
        String::from("  ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compact {}

impl SassOutputStyle for Compact {
    fn brace_property_separator(&self) -> String {
        String::from(" ")
    }

    fn before_property(&self, _nesting: &str) -> String {
        String::from("")
    }

    fn after_property(&self) -> String {
        String::from(" ")
    }

    fn property_brace_separator(&self) -> String {
        String::from(" ")
    }

    fn rule_and_child_rules_separator(&self, _nesting: &str) -> String {
        String::from("\n")
    }

    fn property(&self, name: &str, value: &str) -> String {
        format!("{}: {};", name, value)
    }

    fn comment(&self, content: &str) -> String {
        String::from(content.lines()
                            .map(|s| s.trim())
                            .collect::<Vec<_>>()
                            .join(" "))
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compressed {}

impl SassOutputStyle for Compressed {
    fn rule_separator(&self) -> String {
        String::from("")
    }

    fn selector_separator(&self) -> String {
        String::from(",")
    }

    fn selector_brace_separator(&self) -> String {
        String::from("")
    }

    fn before_property(&self, _nesting: &str) -> String {
        String::from("")
    }

    fn after_property(&self) -> String {
        String::from(";")
    }

    fn child_rule_separator(&self, _has_properties: bool) -> String {
        String::from("")
    }

    fn property(&self, name: &str, value: &str) -> String {
        format!("{}:{}", name, value)
    }

    fn comment(&self, _content: &str) -> String {
        String::new()
    }

    fn after_comment(&self) -> String {
        String::new()
    }

    fn selector_string(&self, rule: &SassRule, parents: &str) -> String {
        let separator: String = self.selector_separator();
        let s = if parents.is_empty() {
            rule.selectors.iter()
                          .map(|s| s.token.to_string())
                          .collect::<Vec<_>>().join(&separator)
        } else {
            parents.split(",").map(|p| {
                rule.selectors.iter().map(|s| {
                    if s.token.to_string().contains("&") {
                        s.token.to_string().replace("&", p.trim())
                    } else {
                        format!("{} {}", p.trim(), s.token)
                    }
                }).collect::<Vec<_>>().join(&separator)
            }).collect::<Vec<_>>().join(&separator)
        };
        self.compress_selectors(s)
    }

    fn filter_child_properties(&self, children: &[Node])
       -> Vec<Node> {
        children.iter().filter(|p|
            match **p {
                Node::Comment(..) => false,
                _ => true,
            }
        ).cloned().collect()
    }
}

impl Compressed {
    fn compress_selectors(&self, selector_string: String) -> String {
        selector_string.replace(" > ", ">").replace(" + ", "+")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Debug {}

impl SassOutputStyle for Debug {
    fn property(&self, name: &str, value: &str) -> String {
        format!("Property({}, {})", name, value)
    }
}
