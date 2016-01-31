use sass::output_style::SassOutputStyle;
use ast::node::Node;
use token::{Lexeme, Token};
use error::Result;

use std::fmt;
use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct SassRule {
    pub selectors: Vec<Lexeme>,
    pub children: Vec<Node>,
}

impl SassRule {
    pub fn new() -> SassRule {
        SassRule {
            selectors: Vec::new(),
            children: Vec::new(),
        }
    }

    fn selector_distribution(&self, parents: &str, separator: &str) -> String {
        match parents.len() {
            0 => self.selectors.iter().map(|s| s.token.to_string()).collect::<Vec<_>>().join(separator),
            _ => parents.split(",").map(|p| {
                self.selectors.iter().map(|s| {
                    if s.token.to_string().contains("&") {
                        s.token.to_string().replace("&", p.trim())
                    } else {
                        format!("{} {}", p.trim(), s.token)
                    }
                }).collect::<Vec<_>>().join(separator)
            }).collect::<Vec<_>>().join(separator),
        }
    }

    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        try!(self.recursive_stream(output, style, "", ""));
        Ok(try!(write!(output, "{}", style.rule_separator())))
    }

    pub fn recursive_stream<W: Write>(&self, output: &mut W, style: SassOutputStyle, parents: &str, nesting: &str) -> Result<()> {
        let mut selector_string = self.selector_distribution(parents, &style.selector_separator());
        if style == SassOutputStyle::Compressed {
            selector_string = compress_selectors(selector_string);
        }

        let mut properties = self.child_properties().into_iter();
        // let mut has_properties = false;

        // TODO: peek?
        if let Some(prop) = properties.next() {
            // has_properties = true;
            try!(write!(output, "{}{}{{{}{}",
              selector_string,
              style.selector_brace_separator(),
              style.brace_property_separator(),
              style.before_property(nesting),
            ));

            try!(prop.stream(output, style));

            for prop in properties {
                try!(write!(output, "{}{}",
                    style.after_property(),
                    style.before_property(nesting),
                ));
                try!(prop.stream(output, style));
            }
            try!(write!(output, "{}}}", style.property_brace_separator()));
        }

        // let mut child_rules = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
        //     match c {
        //         &Event::Rule(ref rule) => rule,
        //         _ => unreachable!(),
        //     }
        // });
        //
        // if let Some(child_rule) = child_rules.next() {
        //     let mut recursive_nesting = String::from(nesting);
        //     if has_properties {
        //         recursive_nesting.push_str("  ");
        //         try!(write!(output, "{}", style.rule_and_child_rules_separator(&recursive_nesting)));
        //     }
        //     try!(child_rule.recursive_stream(output, style, &selector_string, &recursive_nesting));
        //     for child_rule in child_rules {
        //         try!(write!(output, "{}", style.child_rule_separator(has_properties)));
        //         try!(child_rule.recursive_stream(output, style, &selector_string, &recursive_nesting));
        //     }
        // }
        Ok(())
    }

    // TODO: I don't like how child_properties and child_rules are so different, but
    // i'm not sure what to do about it yet.
    pub fn child_properties(&self) -> Vec<&Node> {
        self.children.iter().filter(|c|
            match **c {
                Node::Rule(..) => false,
                Node::Property(..) => true,
            }
        ).collect::<Vec<_>>()
    }

    pub fn child_rules(&self) -> Vec<SassRule> {
        self.children.clone().into_iter().filter_map(|c|
            match c {
                Node::Rule(rule) => Some(rule),
                Node::Property(..) => None,
            }
        ).collect::<Vec<_>>()
    }

    pub fn has_properties(&self) -> bool {
        !self.child_properties().is_empty()
    }

    pub fn optimize(self) -> Vec<SassRule> {
        match self.has_properties() {
            true  => vec![self],
            false => {
                self.child_rules().into_iter().map(|cr|
                    cr.collapse_with_parent_selectors(&self.selectors)
                ).collect()
            },
        }
    }

    pub fn collapse_with_parent_selectors(self, parents: &Vec<Lexeme>) -> SassRule {
        let new_selectors = parents.iter().flat_map(|p|
            self.selectors.iter().map(|c|
                Lexeme {
                    token: Token::Ident(format!("{} {}", p.token, c.token)),
                    offset: p.offset,
                }
            ).collect::<Vec<_>>()
        ).collect();
        SassRule {
            selectors: new_selectors,
            children: self.children,
        }
    }
}

fn compress_selectors(selector_string: String) -> String {
    selector_string.replace(" > ", ">").replace(" + ", "+")
}

impl fmt::Debug for SassRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children = self.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join("\n");
        let indented_children = children.split("\n").collect::<Vec<_>>().join("\n  ");
        write!(f, "{:?} {{\n  {}\n}}", self.selectors, indented_children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::node::Node;
    use ast::expression::Expression;
    use token::{Token, Lexeme};

    #[test]
    fn it_collapses_subrules_without_properties() {
        let orig = SassRule {
            selectors: vec![Lexeme { token: Token::Ident("div".into()), offset: Some(0) }],
            children: vec![Node::Rule(
                SassRule {
                    selectors: vec![Lexeme {
                        token: Token::Ident("img".into()),
                        offset: Some(6)
                    }],
                    children: vec![Node::Property(
                        Lexeme { token: Token::Ident("color".into()), offset: Some(12) },
                        Expression::String(
                            Lexeme { token: Token::Ident("blue".into()), offset: Some(19) }
                        ),
                    )],
                }
            )],
        };

        assert_eq!(
            orig.optimize(),
            vec![SassRule {
                selectors: vec![
                    Lexeme { token: Token::Ident("div img".into()), offset: Some(0) },
                ],
                children: vec![Node::Property(
                    Lexeme { token: Token::Ident("color".into()), offset: Some(12) },
                    Expression::String(
                        Lexeme { token: Token::Ident("blue".into()), offset: Some(19) }
                    ),
                )],
            }]
        );
    }
}
