use sass::value_part::ValuePart;
use sass::op::Op;
use value_tokenizer::ValueTokenizer;

use std::borrow::Cow::Borrowed;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Evaluator<'a> {
    original: &'a str,
    variables: &'a HashMap<String, String>,
    value_stack: Vec<ValuePart<'a>>,
    op_stack: Vec<Op>,
}

impl<'a> Evaluator<'a> {
    pub fn new(original: &'a str, variables: &'a HashMap<String, String>) -> Evaluator<'a> {
        Evaluator {
            original: &original,
            variables: &variables,
            value_stack: Vec::new(),
            op_stack: Vec::new(),
        }
    }

    pub fn evaluate(&mut self) -> String {
        let mut vt = ValueTokenizer::new(self.original);
        let mut last_was_an_operator = true;

        while let Some(part) = vt.next() {
            match part {
                ValuePart::Variable(name) => {
                    match (*self.variables).get(&(*name).to_string()) {
                        Some(ref v) => self.value_stack.push(ValuePart::String(Borrowed(v))),
                        None => self.value_stack.push(ValuePart::String(name)),
                    };
                    last_was_an_operator = false;
                },
                s @ ValuePart::String(..) => {
                    self.value_stack.push(s);
                    last_was_an_operator = false;
                },
                n @ ValuePart::Number(..) => {
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
                        let mut last_operator = self.op_stack.last().expect("Ran out of operators looking for a left paren!").clone();
                        while last_operator != Op::LeftParen {
                            self.math_machine();
                            last_operator = self.op_stack.last().expect("Ran out of operators looking for a left paren!").clone();
                        }
                        self.op_stack.pop();
                    } else if *o == Op::LeftParen {
                        self.op_stack.push(*o);
                    } else {
                        if let Some(&last_operator) = self.op_stack.last() {
                            if last_operator.same_or_greater_precedence(*o) {
                                self.math_machine();
                            }
                        }
                        self.op_stack.push(*o);
                    }
                    last_was_an_operator = true;
                },
                _ => unreachable!(),
            }
        }

        while !self.op_stack.is_empty() {
            self.math_machine();
        }

        // TODO: figure out how to make borrowck like not cloning this, it's cool i swear
        let vs = self.value_stack.clone().into_iter();
        vs.map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
    }

    fn math_machine(&mut self) {
        let op = self.op_stack.pop().expect("No operator on the operator stack!");
        let second = self.value_stack.pop().expect("No second operand on the value stack!");
        let first  = self.value_stack.pop().expect("No first operand on the value stack!");
        self.value_stack.push(op.apply(first, second));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn it_subtitutes_variable_values() {
        let mut vars = HashMap::new();
        vars.insert("$bar".to_string(), "4".to_string());
        vars.insert("$quux".to_string(), "3px 10px".to_string());

        let answer = Evaluator::new("foo $bar 199.82 baz $quux", &vars).evaluate();
        assert_eq!("foo 4 199.82 baz 3px 10px", answer);
    }

    #[test]
    fn it_adds() {
        let answer = Evaluator::new("1 + 2", &HashMap::new()).evaluate();
        assert_eq!("3", answer);
    }

    #[test]
    fn it_doesnt_need_space_around_operators() {
        let answer = Evaluator::new("12*4", &HashMap::new()).evaluate();
        assert_eq!("48", answer);
    }

    #[test]
    fn it_divides_and_adds_with_the_right_precedence() {
        let answer = Evaluator::new("3 + 3/4", &HashMap::new()).evaluate();
        assert_eq!("3.75", answer);
    }

    #[test]
    fn it_gets_the_right_precedence_with_parens() {
        let answer = Evaluator::new("(3 + 3)/4", &HashMap::new()).evaluate();
        assert_eq!("1.5", answer);
    }

    #[test]
    fn it_does_string_concat_when_adding_to_list() {
        let answer = Evaluator::new("2+(3 4)", &HashMap::new()).evaluate();
        assert_eq!("23 4", answer);
    }

    #[test]
    fn it_divides_because_parens_and_string_concats_because_list() {
        let answer = Evaluator::new("1 + (5/10 2 3)", &HashMap::new()).evaluate();
        assert_eq!("10.5 2 3", answer);
    }
}