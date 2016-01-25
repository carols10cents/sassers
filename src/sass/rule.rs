use sass::output_style::SassOutputStyle;
use parser::ASTNode;
use token::Lexeme;
use error::Result;

use std::fmt;
use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct SassRule {
    pub selectors: Vec<Lexeme>,
    pub children: Vec<ASTNode>,
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

        let mut properties = self.children.iter();
        let mut has_properties = false;

        // TODO: peek?
        if let Some(prop) = properties.next() {
            has_properties = true;
            try!(write!(output, "{}{}{{{}{}{}",
              selector_string,
              style.selector_brace_separator(),
              style.brace_property_separator(),
              style.before_property(nesting),
              prop.output(style),
            ));
            for prop in properties {
                try!(write!(output, "{}{}{}",
                    style.after_property(),
                    style.before_property(nesting),
                    prop.output(style),
                ));
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
