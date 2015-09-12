use sass::value_part::ValuePart;
use sass::number_value::NumberValue;
use sass::op::Op;
use value_tokenizer::ValueTokenizer;

use std::collections::HashMap;
extern crate collections;

#[derive(Debug)]
pub struct Evaluator<'a, T> {
    value_tokens: T,
    value_stack: Vec<ValuePart<'a>>,
    op_stack: Vec<Op>,
    paren_level: i32,
}

impl<'a> Evaluator<'a, ValueTokenizer<'a>> {
    pub fn new_from_string(original: &'a str) -> Evaluator<'a, ValueTokenizer<'a>>
    {
        Evaluator {
            value_tokens: ValueTokenizer::new(&original),
            value_stack: Vec::new(),
            op_stack: Vec::new(),
            paren_level: 0,
        }
    }
}

impl<'a> Evaluator<'a, collections::vec::IntoIter<ValuePart<'a>>> {
    pub fn new(value_tokens: Vec<ValuePart<'a>>) -> Evaluator<'a, collections::vec::IntoIter<ValuePart<'a>>>
    {
        Evaluator {
            value_tokens: value_tokens.into_iter(),
            value_stack: Vec::new(),
            op_stack: Vec::new(),
            paren_level: 0,
        }
    }
}

impl<'a, T> Evaluator<'a, T>
where T: Iterator<Item = ValuePart<'a>>
{
    pub fn evaluate(&mut self, variables: &HashMap<String, ValuePart<'a>>) -> ValuePart<'a> {
        let mut last_was_an_operator = true;

        while let Some(part) = self.value_tokens.next() {
            match part {
                ValuePart::Variable(name) => {
                    match variables.get(&(*name).to_string()) {
                        Some(v) => {
                            self.value_stack.push(match v.clone() {
                                ValuePart::Number(nv) => {
                                    ValuePart::Number(NumberValue { computed: true, ..nv })
                                },
                                other => other,
                            })
                        },
                        None => self.value_stack.push(ValuePart::String(name)),
                    }
                    last_was_an_operator = false;
                },
                s @ ValuePart::String(..) => {
                    if last_was_an_operator {
                        self.value_stack.push(s);
                    } else {
                        self.push_on_list_on_value_stack(s);
                    }
                    last_was_an_operator = false;
                },
                n @ ValuePart::Number(..) => {
                    if last_was_an_operator {
                        self.value_stack.push(n);
                    } else {
                        while !self.op_stack.is_empty() &&
                              self.op_stack.last() != Some(&Op::LeftParen) {
                            self.math_machine();
                        }
                        self.push_on_list_on_value_stack(n);
                    }
                    last_was_an_operator = false;
                },
                ValuePart::Operator(ref o) => {
                    if *o == Op::RightParen {
                        let mut last_operator = self.op_stack.last().unwrap_or(&Op::LeftParen).clone();
                        while last_operator != Op::LeftParen {
                            self.math_machine();
                            last_operator = self.op_stack.last().unwrap_or(&Op::LeftParen).clone();
                        }
                        self.op_stack.pop();
                        last_was_an_operator = false;
                        self.paren_level -= 1;
                    } else if *o == Op::LeftParen {
                        self.op_stack.push(*o);
                        last_was_an_operator = true;
                        self.paren_level += 1;
                    } else {
                        if let Some(&last_operator) = self.op_stack.last() {
                            if last_operator.same_or_greater_precedence(*o) {
                                self.math_machine();
                            }
                        }
                        self.op_stack.push(*o);
                        last_was_an_operator = true;
                    }
                },
                list @ ValuePart::List(..) => {
                    self.value_stack.push(list);
                    last_was_an_operator = false;
                },
            }
        }

        while !self.op_stack.is_empty() {
            self.math_machine();
        }

        if self.value_stack.len() > 1 {
            // TODO: figure out how to make borrowck like not cloning this, it's cool i swear
            ValuePart::List(self.value_stack.clone())
        } else {
            self.value_stack.pop().unwrap_or(ValuePart::Number(NumberValue::from_scalar(0.0)))
        }
    }

    fn push_on_list_on_value_stack(&mut self, push_val: ValuePart<'a>) {
        let list_starter = self.value_stack.pop().unwrap_or(ValuePart::List(vec![]));
        let list_parts = match list_starter {
            ValuePart::List(mut v) => {
                v.push(push_val);
                v
            },
            other => vec![other, push_val],
        };
        self.value_stack.push(ValuePart::List(list_parts));
    }

    fn math_machine(&mut self) {
        let op = self.op_stack.pop().unwrap_or(Op::Plus);
        let second = self.value_stack.pop().unwrap_or(ValuePart::Number(NumberValue::from_scalar(0.0)));
        let first  = self.value_stack.pop().unwrap_or(ValuePart::Number(NumberValue::from_scalar(0.0)));
        self.value_stack.push(op.apply(first, second, self.paren_level));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::value_part::ValuePart;
    use sass::number_value::NumberValue;
    use sass::op::Op;
    use std::collections::HashMap;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_subtitutes_variable_values() {
        let mut vars = HashMap::new();
        vars.insert("$bar".to_string(), ValuePart::Number(NumberValue::from_scalar(4.0)));
        vars.insert("$quux".to_string(), ValuePart::List(vec![
            ValuePart::Number(NumberValue::with_units(3.0, Borrowed("px"))),
            ValuePart::Number(NumberValue::with_units(10.0, Borrowed("px"))),
        ]));

        let answer = Evaluator::new_from_string("foo $bar 199.82 baz $quux").evaluate(&vars);

        assert_eq!(
            ValuePart::List(vec![
                ValuePart::String(Borrowed("foo")),
                ValuePart::Number(NumberValue::computed(4.0)),
                ValuePart::Number(NumberValue::from_scalar(199.82)),
                ValuePart::String(Borrowed("baz")),
                ValuePart::Number(NumberValue::with_units(3.0, Borrowed("px"))),
                ValuePart::Number(NumberValue::with_units(10.0, Borrowed("px"))),
            ]),
            answer
        );
    }

    // #[test]
    // fn it_flattents_lists() {
    //     let answer = Evaluator::new_from_string("80% 90%, 80% 90%, 80% 90%").evaluate(&HashMap::new());
    //     assert_eq!(
    //         ValuePart::List(vec![
    //             ValuePart::Number(NumberValue { scalar: 80.0, unit: Some(Borrowed("%")), computed: true}),
    //             ValuePart::Number(NumberValue { scalar: 90.0, unit: Some(Borrowed("%")), computed: true}),
    //             ValuePart::Operator(Op::Comma),
    //             ValuePart::Number(NumberValue { scalar: 80.0, unit: Some(Borrowed("%")), computed: true}),
    //             ValuePart::Number(NumberValue { scalar: 90.0, unit: Some(Borrowed("%")), computed: true}),
    //             ValuePart::Operator(Op::Comma),
    //             ValuePart::Number(NumberValue { scalar: 80.0, unit: Some(Borrowed("%")), computed: true}),
    //             ValuePart::Number(NumberValue { scalar: 90.0, unit: Some(Borrowed("%")), computed: true}),
    //         ]),
    //         answer
    //     );
    // }

    #[test]
    fn it_divides_if_value_came_from_a_variable() {
        let mut vars = HashMap::new();
        vars.insert("$three".to_string(), ValuePart::Number(NumberValue::computed(3.0)));

        let answer = Evaluator::new_from_string("15 / $three").evaluate(&vars);

        assert_eq!(
            ValuePart::Number(NumberValue::computed(5.0)),
            answer
        );
    }

    #[test]
    fn it_divides_if_a_later_value_came_from_a_variable() {
        let mut vars = HashMap::new();
        vars.insert("$three".to_string(), ValuePart::Number(NumberValue::computed(3.0)));

        let answer = Evaluator::new_from_string("15 / 5 / $three").evaluate(&vars);

        assert_eq!(
            ValuePart::Number(NumberValue::computed(1.0)),
            answer
        );
    }

    #[test]
    fn it_divides_out_units() {
        let mut vars = HashMap::new();
        vars.insert(
            "$three".to_string(),
            ValuePart::Number(NumberValue::with_units(3.0, Borrowed("px")))
        );

        let answer = Evaluator::new_from_string("15px / $three").evaluate(&vars);

        assert_eq!(
            ValuePart::Number(NumberValue::computed(5.0)),
            answer
        );
    }

    // #[test]
    // fn it_does_jacked_stuff() {
    //     let mut vars = HashMap::new();
    //     vars.insert("$stuff".to_string(), ValuePart::List(vec![
    //         ValuePart::Number(NumberValue::computed(1.0)),
    //         ValuePart::Number(NumberValue::computed(2.0)),
    //         ValuePart::Number(NumberValue::computed(3.0)),
    //     ]));
    //     let answer = Evaluator::new_from_string("1/2, $stuff url(\"www.foo.com/blah.png\") blah blah").evaluate(&vars);
    //
    //     assert_eq!(
    //         ValuePart::List(vec![
    //             ValuePart::List(vec![
    //                 ValuePart::Number(NumberValue::from_scalar(1.0)),
    //                 ValuePart::Operator(Op::Slash),
    //                 ValuePart::Number(NumberValue::from_scalar(2.0)),
    //             ]),
    //             ValuePart::Operator(Op::Comma),
    //             ValuePart::List(vec![
    //                 ValuePart::Number(NumberValue::computed(1.0)),
    //                 ValuePart::Number(NumberValue::computed(2.0)),
    //                 ValuePart::Number(NumberValue::computed(3.0)),
    //                 ValuePart::String(Borrowed("url(\"www.foo.com/blah.png\")")),
    //                 ValuePart::String(Borrowed("blah")),
    //                 ValuePart::String(Borrowed("blah")),
    //             ]),
    //         ]),
    //         answer
    //     );
    // }

    #[test]
    fn it_handles_lots_of_parens_and_slashes() {
        let answer = Evaluator::new_from_string(
            "1 + (2 + (3/4 + (4/5 6/7)))"
        ).evaluate(&HashMap::new());
        assert_eq!(
            ValuePart::List(vec![
                ValuePart::String(Borrowed("120.750.8")),
                ValuePart::Number(NumberValue::from_scalar(6.0)),
                ValuePart::Operator(Op::Slash),
                ValuePart::Number(NumberValue::from_scalar(7.0)),
            ]),
            answer
        );
    }

    #[test]
    fn it_handles_a_few_parens_and_slashes() {
        let answer = Evaluator::new_from_string("(4/5 6/7)").evaluate(&HashMap::new());
        assert_eq!(
            ValuePart::List(vec![
              ValuePart::Number(NumberValue::computed(0.8)),
              ValuePart::Number(NumberValue::from_scalar(6.0)),
              ValuePart::Operator(Op::Slash),
              ValuePart::Number(NumberValue::from_scalar(7.0)),
            ]),
            answer
        );
    }

    // Not sure what the correct behavior is here yet.
    // #[test]
    // fn it_handles_variables_and_parens() {
    //     let mut vars = HashMap::new();
    //     vars.insert("$foo".to_string(), ValuePart::List(vec![
    //         ValuePart::List(vec![
    //             ValuePart::Number(NumberValue::computed(4.0)),
    //             ValuePart::Operator(Op::Comma),
    //             ValuePart::Number(NumberValue::computed(5.0)),
    //         ]),
    //         ValuePart::Operator(Op::Comma),
    //         ValuePart::Number(NumberValue::computed(6.0)),
    //     ]));
    //     let answer = Evaluator::new_from_string("3 + $foo").evaluate(&vars);
    //
    //     assert_eq!(
    //         ValuePart::List(vec![
    //             ValuePart::String(Borrowed("34")),
    //             ValuePart::Operator(Op::Comma),
    //             ValuePart::Number(NumberValue::computed(5.0)),
    //             ValuePart::Operator(Op::Comma),
    //             ValuePart::Number(NumberValue::computed(6.0)),
    //         ]),
    //         answer
    //     );
    // }

    #[test]
    fn it_adds() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::from_scalar(1.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(NumberValue::from_scalar(2.0)),
        ]).evaluate(&HashMap::new());
        assert_eq!(ValuePart::Number(NumberValue::computed(3.0)), answer);
    }

    #[test]
    fn it_adds_with_units() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::with_units(1.0, Borrowed("px"))),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(NumberValue::with_units(2.0, Borrowed("px"))),
        ]).evaluate(&HashMap::new());
        assert_eq!(
            ValuePart::Number(
                NumberValue { scalar: 3.0, unit: Some(Borrowed("px")), computed: true }
            ),
            answer
        );
    }

    #[test]
    fn it_divides_and_adds_with_the_right_precedence() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(4.0)),
        ]).evaluate(&HashMap::new());
        assert_eq!(ValuePart::Number(NumberValue::computed(3.75)), answer);
    }

    #[test]
    fn it_gets_the_right_precedence_with_parens() {
        let answer = Evaluator::new(vec![
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::RightParen),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(4.0)),
        ]).evaluate(&HashMap::new());
        assert_eq!(ValuePart::Number(NumberValue::computed(1.5)), answer);
    }

    #[test]
    fn it_does_string_concat_when_adding_to_list() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::from_scalar(2.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Number(NumberValue::from_scalar(4.0)),
            ValuePart::Operator(Op::RightParen),
        ]).evaluate(&HashMap::new());

        assert_eq!(ValuePart::List(vec![
            ValuePart::String(Borrowed("23")),
            ValuePart::Number(NumberValue::from_scalar(4.0))
        ]), answer);
    }

    #[test]
    fn it_does_string_concat_when_adding_to_list_in_a_list() {
        let answer = Evaluator::new(vec![
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(NumberValue::from_scalar(2.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Number(NumberValue::from_scalar(4.0)),
            ValuePart::Operator(Op::RightParen),
            ValuePart::Number(NumberValue::from_scalar(5.0)),
            ValuePart::Operator(Op::RightParen),
        ]).evaluate(&HashMap::new());

        assert_eq!(ValuePart::List(vec![
            ValuePart::String(Borrowed("23")),
            ValuePart::Number(NumberValue::from_scalar(4.0)),
            ValuePart::Number(NumberValue::from_scalar(5.0)),
        ]), answer);
    }

    #[test]
    fn it_divides_because_parens_and_string_concats_because_list() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::from_scalar(1.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(NumberValue::from_scalar(5.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(10.0)),
            ValuePart::Number(NumberValue::from_scalar(2.0)),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::RightParen),
        ]).evaluate(&HashMap::new());

        assert_eq!(ValuePart::List(vec![
            ValuePart::String(Borrowed("10.5")),
            ValuePart::Number(NumberValue::from_scalar(2.0)),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
        ]), answer);
    }

    #[test]
    fn it_does_not_divide_when_slash_is_separating() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::from_scalar(15.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(5.0)),
        ]).evaluate(&HashMap::new());

        assert_eq!(ValuePart::List(vec![
            ValuePart::Number(NumberValue::from_scalar(15.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(3.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(5.0)),
        ]), answer);
    }

    #[test]
    fn it_does_divide_when_other_math_is_involved() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(NumberValue::from_scalar(1.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(2.0)),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(NumberValue::from_scalar(1.0)),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(NumberValue::from_scalar(2.0)),
        ]).evaluate(&HashMap::new());

        assert_eq!(ValuePart::Number(NumberValue::computed(1.0)), answer);
    }
}