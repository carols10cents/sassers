use sass::value_part::ValuePart;
use sass::color_value::ColorValue;
use error::{SassError, ErrorKind, Result};
use sass::parameters::*;

use std::borrow::Cow::{self, Owned, Borrowed};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct SassFunctionCall<'a> {
    pub name: Cow<'a, str>,
    pub arguments: Vec<SassArgument<'a>>,
}

impl<'a> SassFunctionCall<'a> {
    pub fn evaluate(&self, variables: &HashMap<String, ValuePart<'a>>) -> Result<ValuePart<'a>> {
        match self.name {
            Borrowed("rgb") => {
                let params = vec![
                    SassParameter { name: Owned("$red".into()), default: None},
                    SassParameter { name: Owned("$green".into()), default: None},
                    SassParameter { name: Owned("$blue".into()), default: None},
                ];

                let resolved = try!(collate_args_parameters(
                    &params,
                    &self.arguments,
                    variables,
                ));

                Ok(ValuePart::Color(
                    try!(ColorValue::from_variables(&resolved))
                ))
            },
            _ => {
                Err(SassError {
                    kind: ErrorKind::UnknownFunction,
                    message: format!(
                        "Don't know how to evaluate function `{}`",
                        self.name,
                    ),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::color_value::ColorValue;
    use sass::value_part::ValuePart;
    use sass::parameters::SassArgument;
    use std::borrow::Cow::Borrowed;
    use std::collections::HashMap;

    #[test]
    fn it_returns_color_for_rgb() {
        let sfc = SassFunctionCall {
            name: Borrowed("rgb"),
            arguments: vec![
                SassArgument { name: None, value: Borrowed("10") },
                SassArgument { name: None, value: Borrowed("100") },
                SassArgument { name: None, value: Borrowed("73") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Color(ColorValue {
                red: 10, green: 100, blue: 73,
                computed: true, original: Borrowed("rgb(10, 100, 73)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }
}
