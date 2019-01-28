use crate::ast::root::Root;
use crate::sass::output_style::Streamable;

pub fn optimize(root: Root) -> Vec<Box<Streamable>> {
    match root {
        Root::Rule(rule) => {
            let mut result: Vec<Box<Streamable>> = Vec::new();
            for r in rule.optimize().into_iter() {
                result.push(Box::new(Root::Rule(r)));
            }
            result
        }
        Root::Comment(c) => vec![Box::new(Root::Comment(c))],
        Root::Variable(..) => unreachable!(), // variables get evaluated before optimization
    }
}
