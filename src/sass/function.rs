use sass::value_part::ValuePart;
use sass::color_value::*;
use sass::number_value::NumberValue;
use error::{SassError, ErrorKind, Result};
use sass::parameters::*;
use token::Token;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct SassFunctionCall {
    pub name: Token,
    pub arguments: Vec<SassArgument>,
}

impl SassFunctionCall {
    pub fn evaluate(&self, variables: &HashMap<Token, ValuePart>) -> Result<ValuePart> {
        match &self.name.value[..] {
            "rgb" => {
                let params = vec![
                    SassParameter::new("$red"),
                    SassParameter::new("$green"),
                    SassParameter::new("$blue"),
                ];

                debug!("collate_args_parameters 26");
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
                            SassParameter::new("$red"),
                            SassParameter::new("$green"),
                            SassParameter::new("$blue"),
                            SassParameter::new("$alpha"),
                        ];

                        debug!("collate_args_parameters 47");
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
                            SassParameter::new("$color"),
                            SassParameter::new("$alpha"),
                        ];

                        debug!("collate_args_parameters 64");
                        let resolved = try!(collate_args_parameters(
                            &params,
                            &self.arguments,
                            variables,
                        ));

                        let color_token = Token {
                            value: "$color".into(),
                            offset: None,
                        };

                        let color = try!(match resolved.get(&color_token) {
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
                    SassParameter::new("$color1"),
                    SassParameter::new("$color2"),
                ];

                debug!("collate_args_parameters 111");
                let resolved = try!(collate_args_parameters(
                    &params,
                    &self.arguments,
                    variables,
                ));

                let color1 = "$color1".into();
                let color2 = "$color2".into();

                match (resolved.get(&color1), resolved.get(&color2)) {
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
                    SassParameter::new("$value"),
                ];
                debug!("collate_args_parameters 140");
                let resolved = try!(collate_args_parameters(
                    &params,
                    &self.arguments,
                    variables,
                ));
                let value_token = Token { value: "$value".into(), offset: None };
                let t = match resolved.get(&value_token) {
                    Some(&ValuePart::Color(..)) => Token {
                        value: String::from("color"),
                        offset: None,
                    },
                    Some(&ValuePart::Number(..)) => Token {
                        value: String::from("number"),
                        offset: None,
                    },
                    Some(&ValuePart::String(..)) => Token {
                        value: String::from("string"),
                        offset: None,
                    },
                    other => return Err(SassError {
                        offset: 0,
                        kind: ErrorKind::UnexpectedValuePartType,
                        message: format!(
                            "Don't know type-of value `{:?}`", other
                        )
                    }),
                };
                Ok(ValuePart::String(t))
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

    fn extract_color_part(&self, which_color: &str, variables: &HashMap<Token, ValuePart>) -> Result<ValuePart> {
        let params = vec![
            SassParameter::new("$color"),
        ];
        debug!("collate_args_parameters 196");
        let resolved = try!(collate_args_parameters(
            &params,
            &self.arguments,
            variables,
        ));
        let color_token = Token {
            value: "$color".into(),
            offset: None,
        };

        match resolved.get(&color_token) {
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
            name: "rgb".into(),
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
            name: "rgba".into(),
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
            name: "rgba".into(),
            arguments: vec![
                SassArgument::new("#f0e"),
                SassArgument::new("$alpha: .6"),
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
            name: "red".into(),
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
