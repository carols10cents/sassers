use ast::root::Root;
use sass::variable::SassVariable;
use context::Context;

pub fn evaluate(root: Root, context: &mut Context) -> Option<Root> {
    match root {
        Root::Rule(sr) => Some(Root::Rule(sr.evaluate(&context))),
        Root::Variable(sv) => {
            let evaluated_var = sv.value.evaluate(&context);
            context.add_variable(SassVariable {
                name: sv.name,
                value: evaluated_var,
            });
            None
        },
    }
}
