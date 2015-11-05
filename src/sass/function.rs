use sass::value_part::ValuePart;
use sass::color_value::*;
use sass::number_value::NumberValue;
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
            Borrowed("rgba") => {
                match self.arguments.len() {
                    4 => {
                        let params = vec![
                            SassParameter { name: Owned("$red".into()), default: None},
                            SassParameter { name: Owned("$green".into()), default: None},
                            SassParameter { name: Owned("$blue".into()), default: None},
                            SassParameter { name: Owned("$alpha".into()), default: None},
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
                    2 => {
                        let params = vec![
                            SassParameter { name: Owned("$color".into()), default: None},
                            SassParameter { name: Owned("$alpha".into()), default: None},
                        ];

                        let resolved = try!(collate_args_parameters(
                            &params,
                            &self.arguments,
                            variables,
                        ));

                        let color = try!(match resolved.get("$color".into()) {
                            Some(&ValuePart::Color(ref cv)) => Ok(cv.clone()),
                            Some(&ValuePart::String(ref s)) => ColorValue::from_hex(s.clone()),
                            ref e @ Some(_) | ref e @ None => Err(SassError {
                                kind: ErrorKind::UnexpectedValuePartType,
                                message: format!(
                                    "Expected color argument to rgba to be a color-like ValuePart; instead got `{:?}`", e
                                )
                            }),
                        });
                        let alpha = try!(alpha_from_variables(&resolved));

                        Ok(ValuePart::Color(
                            ColorValue::from_color_and_alpha(
                                color, alpha
                            )
                        ))
                    },
                    _ => Err(SassError {
                        kind: ErrorKind::WrongNumberOfArguments,
                        message: format!(
                            "Expected 2 or 4 arguments to rgba; got {}: `{:?}`",
                            self.arguments.len(), self.arguments
                        ),
                    })
                }
            },
            Borrowed("mix") => {
                let params = vec![
                    SassParameter { name: Owned("$color1".into()), default: None},
                    SassParameter { name: Owned("$color2".into()), default: None},
                ];

                let resolved = try!(collate_args_parameters(
                    &params,
                    &self.arguments,
                    variables,
                ));

                match (resolved.get("$color1".into()), resolved.get("$color2".into())) {
                    (Some(&ValuePart::Color(ref cv1)), Some(&ValuePart::Color(ref cv2))) => {
                        Ok(ValuePart::Color(try!(cv1.mix(cv2))))
                    },
                    (other1, other2) => {
                        Err(SassError {
                            kind: ErrorKind::UnexpectedValuePartType,
                            message: format!(
                                "Expected arguments to mix to be Colors; instead got `{:?}` and `{:?}`", other1, other2
                            )
                        })
                    }
                }
            },
            Borrowed("red") => {
                self.extract_color_part("red", variables)
            },
            Borrowed("green") => {
                self.extract_color_part("green", variables)
            },
            Borrowed("blue") => {
                self.extract_color_part("blue", variables)
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

    fn extract_color_part(&self, which_color: &str, variables: &HashMap<String, ValuePart<'a>>) -> Result<ValuePart<'a>> {
        let params = vec![
            SassParameter { name: Owned("$color".into()), default: None},
        ];
        let resolved = try!(collate_args_parameters(
            &params,
            &self.arguments,
            variables,
        ));

        match resolved.get("$color".into()) {
            Some(&ValuePart::Color(ref cv)) => {
                let extracted = match which_color {
                    "red" => cv.red,
                    "green" => cv.green,
                    "blue" => cv.blue,
                    _ => unreachable!(),
                };
                Ok(ValuePart::Number(NumberValue::from_scalar(extracted as f32)))
            },
            ref e => Err(SassError {
                kind: ErrorKind::UnexpectedValuePartType,
                message: format!(
                    "Expected color argument to {} to be a Color; instead got `{:?}`", which_color, e
                )
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::color_value::ColorValue;
    use sass::value_part::ValuePart;
    use sass::number_value::NumberValue;
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
                alpha: None,
                computed: true, original: Borrowed("rgb(10, 100, 73)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }

    #[test]
    fn it_returns_color_with_alpha_for_rgba() {
        let sfc = SassFunctionCall {
            name: Borrowed("rgba"),
            arguments: vec![
                SassArgument { name: None, value: Borrowed("10") },
                SassArgument { name: None, value: Borrowed("100") },
                SassArgument { name: None, value: Borrowed("73") },
                SassArgument { name: None, value: Borrowed(".5") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Color(ColorValue {
                red: 10, green: 100, blue: 73, alpha: Some(0.5),
                computed: true, original: Borrowed("rgba(10, 100, 73, 0.5)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }

    #[test]
    fn it_returns_color_with_alpha_from_color_argument_for_rgba() {
        let sfc = SassFunctionCall {
            name: Borrowed("rgba"),
            arguments: vec![
                SassArgument { name: None, value: Borrowed("#f0e") },
                SassArgument { name: Some(Borrowed("$alpha")), value: Borrowed(".6") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Color(ColorValue {
                red: 255, green: 0, blue: 238, alpha: Some(0.6),
                computed: true, original: Borrowed("rgba(255, 0, 238, 0.6)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }

    #[test]
    fn it_returns_red_part_of_color_for_red() {
        let sfc = SassFunctionCall {
            name: Borrowed("red"),
            arguments: vec![
                SassArgument { name: None, value: Borrowed("#cba") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Number(NumberValue::from_scalar(204.0))),
            sfc.evaluate(&HashMap::new())
        );
    }
}
