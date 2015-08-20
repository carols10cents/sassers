use sass::value_part::ValuePart;
use sass::op::Op;
use value_tokenizer::ValueTokenizer;

use std::borrow::Cow::Borrowed;
use std::collections::HashMap;
extern crate collections;

#[derive(Debug)]
pub struct Evaluator<'a, T> {
    value_tokens: T,
    variables: Option<&'a HashMap<String, String>>,
    value_stack: Vec<ValuePart<'a>>,
    op_stack: Vec<Op>,
    paren_level: i32,
}

impl<'a> Evaluator<'a, ValueTokenizer<'a>> {
    pub fn new_from_string(original: &'a str, variables: &'a HashMap<String, String>) -> Evaluator<'a, ValueTokenizer<'a>>
    {
        Evaluator {
            value_tokens: ValueTokenizer::new(&original),
            variables: Some(&variables),
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
            variables: None,
            value_stack: Vec::new(),
            op_stack: Vec::new(),
            paren_level: 0,
        }
    }
}

impl<'a, T> Evaluator<'a, T>
where T: Iterator<Item = ValuePart<'a>>
{
    pub fn evaluate(&mut self) -> ValuePart<'a> {
        let mut last_was_an_operator = true;

        while let Some(part) = self.value_tokens.next() {
            match part {
                ValuePart::Variable(name) => {
                    match self.variables {
                        Some(vars) => {
                            match vars.get(&(*name).to_string()) {
                                Some(ref v) => self.value_stack.push(ValuePart::String(Borrowed(v))),
                                None => self.value_stack.push(ValuePart::String(name)),
                            }
                        },
                        None => self.value_stack.push(ValuePart::String(name)),
                    }
                    last_was_an_operator = false;
                },
                s @ ValuePart::String(..) => {
                    self.value_stack.push(s);
                    last_was_an_operator = false;
                },
                n @ ValuePart::Number(..) | n @ ValuePart::NumberUnits(..) => {
                    if last_was_an_operator {
                        self.value_stack.push(n);
                    } else {
                        while !self.op_stack.is_empty() && self.op_stack.last() != Some(&Op::LeftParen) {
                            self.math_machine();
                        }
                        let list_starter = self.value_stack.pop().expect("No expected list starter on the value stack!");
                        let list_parts = match list_starter {
                            ValuePart::List(mut v) => {
                                v.push(n);
                                v
                            },
                            other => vec![other, n],
                        };
                        self.value_stack.push(ValuePart::List(list_parts));
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
                _ => unreachable!(),
            }
        }

        while !self.op_stack.is_empty() {
            self.math_machine();
        }

        if self.value_stack.len() > 1 {
            // TODO: figure out how to make borrowck like not cloning this, it's cool i swear
            ValuePart::List(self.value_stack.clone())
        } else {
            self.value_stack.pop().unwrap_or(ValuePart::Number(0.0))
        }
    }

    fn math_machine(&mut self) {
        let op = self.op_stack.pop().unwrap_or(Op::Plus);
        let second = self.value_stack.pop().unwrap_or(ValuePart::Number(0.0));
        let first  = self.value_stack.pop().unwrap_or(ValuePart::Number(0.0));
        self.value_stack.push(op.apply(first, second, self.paren_level));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::value_part::ValuePart;
    use sass::op::Op;
    use std::collections::HashMap;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_subtitutes_variable_values() {
        let mut vars = HashMap::new();
        vars.insert("$bar".to_string(), "4".to_string());
        vars.insert("$quux".to_string(), "3px 10px".to_string());

        let answer = Evaluator::new_from_string("foo $bar 199.82 baz $quux", &vars).evaluate();

        assert_eq!(
            ValuePart::List(vec![
                ValuePart::String(Borrowed("foo")),
                ValuePart::List(vec![
                    ValuePart::String(Borrowed("4")),
                    ValuePart::Number(199.82)
                ]),
                ValuePart::String(Borrowed("baz")),
                ValuePart::String(Borrowed("3px 10px"))
            ]),
            answer
        );
    }

    #[test]
    fn it_adds() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(1.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(2.0),
        ]).evaluate();
        assert_eq!(ValuePart::Computed(3.0), answer);
    }

    #[test]
    fn it_divides_and_adds_with_the_right_precedence() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(4.0),
        ]).evaluate();
        assert_eq!(ValuePart::Computed(3.75), answer);
    }

    #[test]
    fn it_gets_the_right_precedence_with_parens() {
        let answer = Evaluator::new(vec![
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::RightParen),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(4.0),
        ]).evaluate();
        assert_eq!(ValuePart::Computed(1.5), answer);
    }

    #[test]
    fn it_does_string_concat_when_adding_to_list() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(2.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(3.0),
            ValuePart::Number(4.0),
            ValuePart::Operator(Op::RightParen),
        ]).evaluate();

        assert_eq!(ValuePart::List(vec![
            ValuePart::String(Borrowed("23")),
            ValuePart::Number(4.0)
        ]), answer);
    }

    #[test]
    fn it_does_string_concat_when_adding_to_list_in_a_list() {
        let answer = Evaluator::new(vec![
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(2.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(3.0),
            ValuePart::Number(4.0),
            ValuePart::Operator(Op::RightParen),
            ValuePart::Number(5.0),
            ValuePart::Operator(Op::RightParen),
        ]).evaluate();

        assert_eq!(ValuePart::List(vec![
            ValuePart::String(Borrowed("23")),
            ValuePart::Number(4.0),
            ValuePart::Number(5.0)
        ]), answer);
    }

    #[test]
    fn it_divides_because_parens_and_string_concats_because_list() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(1.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Operator(Op::LeftParen),
            ValuePart::Number(5.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(10.0),
            ValuePart::Number(2.0),
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::RightParen),
        ]).evaluate();

        assert_eq!(ValuePart::List(vec![
            ValuePart::String(Borrowed("10.5")),
            ValuePart::Number(2.0),
            ValuePart::Number(3.0)
        ]), answer);
    }

    #[test]
    fn it_does_not_divide_when_slash_is_separating() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(15.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(5.0),
        ]).evaluate();

        assert_eq!(ValuePart::List(vec![
            ValuePart::Number(15.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(3.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(5.0),
        ]), answer);
    }

    #[test]
    fn it_does_divide_when_other_math_is_involved() {
        let answer = Evaluator::new(vec![
            ValuePart::Number(1.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(2.0),
            ValuePart::Operator(Op::Plus),
            ValuePart::Number(1.0),
            ValuePart::Operator(Op::Slash),
            ValuePart::Number(2.0),
        ]).evaluate();

        assert_eq!(ValuePart::Number(1.0), answer);
    }
}