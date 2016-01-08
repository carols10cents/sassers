use sass::value_part::ValuePart;
use sass::color_value::*;
use sass::number_value::NumberValue;
use error::{SassError, ErrorKind, Result};
use sass::parameters::*;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct SassFunctionCall {
    pub name: String,
    pub arguments: Vec<SassArgument>,
}

impl SassFunctionCall {
    pub fn evaluate(&self, variables: &HashMap<String, ValuePart>) -> Result<ValuePart> {
        match &self.name[..] {
            "rgb" => {
                let params = vec![
                    SassParameter { name: String::from("$red"), default: None},
                    SassParameter { name: String::from("$green"), default: None},
                    SassParameter { name: String::from("$blue"), default: None},
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
            "rgba" => {
                match self.arguments.len() {
                    4 => {
                        let params = vec![
                            SassParameter { name: String::from("$red"), default: None},
                            SassParameter { name: String::from("$green"), default: None},
                            SassParameter { name: String::from("$blue"), default: None},
                            SassParameter { name: String::from("$alpha"), default: None},
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
                            SassParameter { name: String::from("$color"), default: None},
                            SassParameter { name: String::from("$alpha"), default: None},
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
                                offset: 0,
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
                        offset: 0,
                        kind: ErrorKind::WrongNumberOfArguments,
                        message: format!(
                            "Expected 2 or 4 arguments to rgba; got {}: `{:?}`",
                            self.arguments.len(), self.arguments
                        ),
                    })
                }
            },
            "mix" => {
                let params = vec![
                    SassParameter { name: String::from("$color1"), default: None},
                    SassParameter { name: String::from("$color2"), default: None},
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
                            offset: 0,
                            kind: ErrorKind::UnexpectedValuePartType,
                            message: format!(
                                "Expected arguments to mix to be Colors; instead got `{:?}` and `{:?}`", other1, other2
                            )
                        })
                    }
                }
            },
            "type-of" => {
                let params = vec![
                    SassParameter { name: String::from("$value"), default: None},
                ];
                let resolved = try!(collate_args_parameters(
                    &params,
                    &self.arguments,
                    variables,
                ));
                match resolved.get("$value".into()) {
                    Some(&ValuePart::Color(..)) => Ok(ValuePart::String(String::from("color"))),
                    Some(&ValuePart::Number(..)) => Ok(ValuePart::String(String::from("number"))),
                    Some(&ValuePart::String(..)) => Ok(ValuePart::String(String::from("string"))),
                    other => Err(SassError {
                        offset: 0,
                        kind: ErrorKind::UnexpectedValuePartType,
                        message: format!(
                            "Don't know type-of value `{:?}`", other
                        )
                    }),
                }
            },
            "red" => {
                self.extract_color_part("red", variables)
            },
            "green" => {
                self.extract_color_part("green", variables)
            },
            "blue" => {
                self.extract_color_part("blue", variables)
            },
            _ => {
                Err(SassError {
                    offset: 0,
                    kind: ErrorKind::UnknownFunction,
                    message: format!(
                        "Don't know how to evaluate function `{}`",
                        self.name,
                    ),
                })
            }
        }
    }

    fn extract_color_part(&self, which_color: &str, variables: &HashMap<String, ValuePart>) -> Result<ValuePart> {
        let params = vec![
            SassParameter { name: String::from("$color"), default: None},
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
                offset: 0,
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
    use std::collections::HashMap;

    #[test]
    fn it_returns_color_for_rgb() {
        let sfc = SassFunctionCall {
            name: String::from("rgb"),
            arguments: vec![
                SassArgument { name: None, value: String::from("10") },
                SassArgument { name: None, value: String::from("100") },
                SassArgument { name: None, value: String::from("73") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Color(ColorValue {
                red: 10, green: 100, blue: 73,
                alpha: None,
                computed: true, original: String::from("rgb(10, 100, 73)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }

    #[test]
    fn it_returns_color_with_alpha_for_rgba() {
        let sfc = SassFunctionCall {
            name: String::from("rgba"),
            arguments: vec![
                SassArgument { name: None, value: String::from("10") },
                SassArgument { name: None, value: String::from("100") },
                SassArgument { name: None, value: String::from("73") },
                SassArgument { name: None, value: String::from(".5") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Color(ColorValue {
                red: 10, green: 100, blue: 73, alpha: Some(0.5),
                computed: true, original: String::from("rgba(10, 100, 73, 0.5)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }

    #[test]
    fn it_returns_color_with_alpha_from_color_argument_for_rgba() {
        let sfc = SassFunctionCall {
            name: String::from("rgba"),
            arguments: vec![
                SassArgument { name: None, value: String::from("#f0e") },
                SassArgument { name: Some(String::from("$alpha")), value: String::from(".6") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Color(ColorValue {
                red: 255, green: 0, blue: 238, alpha: Some(0.6),
                computed: true, original: String::from("rgba(255, 0, 238, 0.6)"),
            })),
            sfc.evaluate(&HashMap::new())
        );
    }

    #[test]
    fn it_returns_red_part_of_color_for_red() {
        let sfc = SassFunctionCall {
            name: String::from("red"),
            arguments: vec![
                SassArgument { name: None, value: String::from("#cba") },
            ],
        };
        assert_eq!(
            Ok(ValuePart::Number(NumberValue::from_scalar(204.0))),
            sfc.evaluate(&HashMap::new())
        );
    }
}
