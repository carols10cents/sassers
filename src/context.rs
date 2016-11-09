use ast::expression::Expression;
use sass::variable::SassVariable;
use token_offset::TokenOffset;
use token::Token;
use operator_or_token::OperatorOrToken;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context {
    pub variables: HashMap<String, SassVariable>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            variables: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, variable: SassVariable) {
        let computed_var = match variable {
            SassVariable {
                value: Expression::Value(
                    OperatorOrToken::Token(TokenOffset {
                        token: Token::Number { value, units, .. },
                        offset
                    })
                ),
                name
            } => {
                SassVariable {
                    name: name,
                    value: Expression::Value(
                        OperatorOrToken::Token(TokenOffset {
                            token: Token::Number {
                                value: value,
                                units: units,
                                computed: true,
                            },
                            offset: offset,
                        })
                    ),
                }

            },
            other => other,
        };
        self.variables.insert(computed_var.name_string(), computed_var);
    }

    pub fn get_variable(&self, token_offset: &TokenOffset) -> Option<Expression> {
        self.variables.get(
            &token_offset.token.to_string()
        ).and_then( |sv| Some(sv.value.clone()) )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use token_offset::TokenOffset;
    use token::Token;
    use sass::variable::SassVariable;
    use operator_or_token::OperatorOrToken;
    use ast::expression::Expression;

    #[test]
    fn it_sets_number_token_computed_to_true() {
        let name = TokenOffset {
            token: Token::String(String::from("$a")),
            offset: None,
        };
        // Whether variables are computed or not when they're inserted
        // shouldn't matter as long as computed is true on retrieval
        let number = Token::Number {
            value: 1.0, units: None, computed: false,
        };

        let var = SassVariable {
            name: name.clone(),
            value: Expression::Value(OperatorOrToken::Token(
                TokenOffset {
                    token: number.clone(),
                    offset: None,
                }
            ))
        };

        let mut context = Context::new();
        context.add_variable(var);

        let expected_number = Token::Number {
            value: 1.0, units: None, computed: true,
        };

        assert_eq!(
            context.get_variable(&name),
            Some(Expression::Value(OperatorOrToken::Token(TokenOffset {
                token: expected_number,
                offset: None,
            })))
        );
    }
}

