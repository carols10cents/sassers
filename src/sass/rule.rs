use sass::output_style::SassOutputStyle;
use sass::variable::SassVariable;
use ast::node::Node;
use token::{Token, TokenOffset};
use error::Result;
use context::Context;

use std::io::Write;

#[derive(Clone, PartialEq, Debug)]
pub struct SassRule {
    pub selectors: Vec<TokenOffset>,
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
        let mut has_properties = false;

        if style == SassOutputStyle::Compressed {
            properties = properties.filter(|p|
                match **p {
                    Node::Comment(..) => false,
                    _ => true,
                }
            ).collect::<Vec<_>>().into_iter();
        }

        // TODO: peek?
        if let Some(prop) = properties.next() {
            has_properties = true;
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


        let mut child_rules = self.child_rules().into_iter();
        if let Some(cr) = child_rules.next() {
            let mut recursive_nesting = String::from(nesting);
            if has_properties {
                recursive_nesting.push_str("  ");
                try!(write!(
                    output, "{}", style.rule_and_child_rules_separator(&recursive_nesting)
                ));
            }
            try!(cr.recursive_stream(output, style, &selector_string, &recursive_nesting));
            for cr in child_rules {
                try!(write!(output, "{}", style.child_rule_separator(has_properties)));
                try!(cr.recursive_stream(output, style, &selector_string, &recursive_nesting));
            }
        }

        Ok(())
    }

    // TODO: I don't like how child_properties and child_rules are so different, but
    // i'm not sure what to do about it yet.
    pub fn child_properties(&self) -> Vec<&Node> {
        self.children.iter().filter(|c|
            match **c {
                Node::Rule(..)     => false,
                Node::Comment(..)  => true,
                Node::Property(..) => true,
                Node::Variable(..) => true,
            }
        ).collect::<Vec<_>>()
    }

    pub fn child_rules(&self) -> Vec<SassRule> {
        self.children.clone().into_iter().filter_map(|c|
            match c {
                Node::Rule(rule)   => Some(rule),
                Node::Comment(..)  => None,
                Node::Property(..) => None,
                Node::Variable(..) => None,
            }
        ).collect::<Vec<_>>()
    }

    pub fn has_properties(&self) -> bool {
        !self.child_properties().is_empty()
    }

    pub fn optimize(self) -> Vec<SassRule> {
        let mut results = vec![];
        if self.has_properties() {
            results.push(self.clone());
            return results
        } else {
            self.child_rules().into_iter().flat_map(|cr|
                cr.collapse_with_parent_selectors(&self.selectors)
            ).collect::<Vec<_>>()
        }
    }

    pub fn collapse_with_parent_selectors(self, parents: &Vec<TokenOffset>) -> Vec<SassRule> {
        let new_selectors = parents.iter().flat_map(|p|
            self.selectors.iter().map(|c|
                TokenOffset {
                    token: Token::String(format!("{} {}", p.token, c.token)),
                    offset: p.offset,
                }
            ).collect::<Vec<_>>()
        ).collect();
        SassRule {
            selectors: new_selectors,
            children: self.children,
        }.optimize()
    }

    pub fn evaluate(self, context: &Context) -> SassRule {
        let mut local_context = (*context).clone();
        SassRule {
            selectors: self.selectors,
            children: self.children.into_iter().filter_map(|c|
                match c {
                    Node::Rule(sr) => Some(Node::Rule(sr.evaluate(&local_context))),
                    Node::Property(lex, ex) => {
                        Some(Node::Property(lex, ex.evaluate(&local_context)))
                    },
                    Node::Comment(sc) => Some(Node::Comment(sc)),
                    Node::Variable(sv) => {
                        let evaluated_var = sv.value.evaluate(&local_context);
                        local_context.add_variable(SassVariable {
                            name: sv.name,
                            value: evaluated_var,
                        });
                        None
                    },
                }
            ).collect(),
        }
    }
}

fn compress_selectors(selector_string: String) -> String {
    selector_string.replace(" > ", ">").replace(" + ", "+")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::node::Node;
    use ast::expression::Expression;
    use token::{Token, TokenOffset};
    use operator_or_token::OperatorOrToken;

    #[test]
    fn it_collapses_subrules_without_properties() {
        let innermost_rule = SassRule {
            selectors: vec![TokenOffset {
                token: Token::String("strong".into()),
                offset: Some(6)
            }],
            children: vec![
                Node::Property(
                    TokenOffset { token: Token::String("font-weight".into()), offset: Some(12) },
                    Expression::Value(OperatorOrToken::Token(
                        TokenOffset {
                            token: Token::String("bold".into()),
                            offset: Some(19)
                        }
                    )),
                ),
            ],
        };

        let further_in_rule = SassRule {
            selectors: vec![TokenOffset {
                token: Token::String("img".into()),
                offset: Some(6)
            }],
            children: vec![
                Node::Property(
                    TokenOffset { token: Token::String("color".into()), offset: Some(12) },
                    Expression::Value(OperatorOrToken::Token(
                        TokenOffset {
                            token: Token::String("blue".into()),
                            offset: Some(19)
                        }
                    )),
                ),
                Node::Rule(innermost_rule),
            ],
        };

        let middle_rule = SassRule {
            selectors: vec![TokenOffset { token: Token::String("span".into()), offset: Some(0) }],
            children: vec![Node::Rule(further_in_rule)],
        };

        let outer_rule = SassRule {
            selectors: vec![TokenOffset { token: Token::String("div".into()), offset: Some(0) }],
            children: vec![Node::Rule(middle_rule)],
        };

        assert_eq!(
            outer_rule.optimize(),
            vec![
                SassRule {
                    selectors: vec![
                        TokenOffset { token: Token::String("div span img".into()), offset: Some(0) },
                    ],
                    children: vec![
                        Node::Property(
                            TokenOffset { token: Token::String("color".into()), offset: Some(12) },
                            Expression::Value(OperatorOrToken::Token(
                                TokenOffset {
                                    token: Token::String("blue".into()),
                                    offset: Some(19)
                                }
                            )),
                        ),
                        Node::Rule(
                            SassRule {
                                selectors: vec![
                                    TokenOffset {
                                        token: Token::String("strong".into()),
                                        offset: Some(6)
                                    },
                                ],
                                children: vec![Node::Property(
                                    TokenOffset { token: Token::String("font-weight".into()), offset: Some(12) },
                                    Expression::Value(OperatorOrToken::Token(
                                        TokenOffset {
                                            token: Token::String(
                                                "bold".into()
                                            ),
                                            offset: Some(19)
                                        }
                                    )),
                                )],
                            }
                        ),
                    ],
                },
            ]
        );
    }
}
