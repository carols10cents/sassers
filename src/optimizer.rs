use ast::root::Root;

pub fn optimize(root: Root) -> Vec<Root> {
    match root {
        Root::Rule(rule) => {
            rule.optimize().into_iter().map(|r| Root::Rule(r) ).collect()
        },
        Root::Variable(..) => unreachable!(), // variables get evaluated before optimization
    }
}
