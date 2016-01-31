use ast::root::Root;

pub fn optimize(root: Root) -> Root {
    root
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::node::Node;
    use ast::root::Root;
    use ast::expression::Expression;
    use token::{Token, Lexeme};
    use sass::rule::SassRule;

    #[test]
    fn it_collapses_subrules_without_properties() {
        let orig = Root::Rule(SassRule {
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
        });

        assert_eq!(
            optimize(orig),
            Root::Rule(SassRule {
                selectors: vec![
                    Lexeme { token: Token::Ident("div img".into()), offset: Some(0) },
                ],
                children: vec![Node::Property(
                    Lexeme { token: Token::Ident("color".into()), offset: Some(12) },
                    Expression::String(
                        Lexeme { token: Token::Ident("blue".into()), offset: Some(19) }
                    ),
                )],
            })
        );
    }
}
