use ast::expression::Expression;
use ast::root::Root;
use token::{Token, Lexeme};
use context::Context;

use std::collections::HashMap;

pub fn evaluate(root: Root, context: &mut Context) -> Option<Root> {
    match root {
        Root::Rule(..) => Some(root),
        _ => None,
    }
}

// use error::{Result, SassError, ErrorKind};
// use sass::value_part::*;
// use sass::number_value::NumberValue;
// use sass::op::Op;
// use value_tokenizer::ValueTokenizer;
// use token::Token;
//
// use std::collections::HashMap;
// use std::vec;
//
// #[derive(Debug)]
// pub struct Evaluator<T> {
//     value_tokens: T,
//     value_stack: Vec<ValuePart>,
//     op_stack: Vec<Op>,
//     paren_level: i32,
// }
//
// impl<'a> Evaluator<ValueTokenizer<'a>> {
//     pub fn new_from_string(original: &'a str) -> Evaluator<ValueTokenizer<'a>>
//     {
//         Evaluator {
//             value_tokens: ValueTokenizer::new(original),
//             value_stack: Vec::new(),
//             op_stack: Vec::new(),
//             paren_level: 0,
//         }
//     }
// }
//
// impl Evaluator<vec::IntoIter<Result<ValuePart>>> {
//     pub fn new(value_tokens: Vec<Result<ValuePart>>) -> Evaluator<vec::IntoIter<Result<ValuePart>>>
//     {
//         Evaluator {
//             value_tokens: value_tokens.into_iter(),
//             value_stack: Vec::new(),
//             op_stack: Vec::new(),
//             paren_level: 0,
//         }
//     }
// }
//
// impl<T> Evaluator<T>
// where T: Iterator<Item = Result<ValuePart>>
// {
//     pub fn evaluate(&mut self, variables: &HashMap<Token, ValuePart>) -> Result<ValuePart> {
//         debug!("=====new evaluate call=======");
//         let mut last_was_an_operator = true;
//
//         while let Some(part) = self.value_tokens.next() {
//             debug!("evaluate processing {:?}", part);
//             match part {
//                 Ok(ValuePart::Variable(name)) => {
//                     debug!("matched a variable: {:?}, variables: {:?}", name, variables);
//                     let value = match variables.get(&name) {
//                         Some(v) => {
//                             debug!("evaluate substituting Variable {:?} with {:?}", name, v);
//                             match v.clone() {
//                                 ValuePart::Number(nv) => {
//                                     ValuePart::Number(NumberValue { computed: true, ..nv })
//                                 },
//                                 other => other,
//                             }
//                         },
//                         None => ValuePart::String(name),
//                     };
//                     if last_was_an_operator {
//                         self.value_stack.push(value);
//                     } else {
//                         let _ = self.push_on_list_on_value_stack(value);
//                     }
//                     last_was_an_operator = false;
//                 },
//                 Ok(ValuePart::String(s)) => {
//                     if last_was_an_operator {
//                         self.value_stack.push(ValuePart::String(s));
//                     } else {
//                         let _ = self.push_on_list_on_value_stack(ValuePart::String(s));
//                     }
//                     last_was_an_operator = false;
//                 },
//                 Ok(n @ ValuePart::Number(..)) => {
//                     if last_was_an_operator {
//                         self.value_stack.push(n);
//                     } else {
//                         while !self.op_stack.is_empty() &&
//                               self.op_stack.last() != Some(&Op::LeftParen) {
//                             let _ = self.math_machine();
//                         }
//                         let _ = self.push_on_list_on_value_stack(n);
//                     }
//                     last_was_an_operator = false;
//                 },
//                 Ok(ValuePart::Operator(ref o)) => {
//                     if *o == Op::RightParen {
//                         let mut last_operator = self.op_stack.last().unwrap_or(&Op::LeftParen).clone();
//                         while last_operator != Op::LeftParen {
//                             let _ = self.math_machine();
//                             last_operator = self.op_stack.last().unwrap_or(&Op::LeftParen).clone();
//                         }
//                         self.op_stack.pop();
//                         last_was_an_operator = false;
//                         self.paren_level -= 1;
//                     } else if *o == Op::LeftParen {
//                         self.op_stack.push(*o);
//                         last_was_an_operator = true;
//                         self.paren_level += 1;
//                     } else {
//                         if let Some(&last_operator) = self.op_stack.last() {
//                             if last_operator.same_or_greater_precedence(*o) {
//                                 let _ = self.math_machine();
//                             }
//                         }
//                         self.op_stack.push(*o);
//                         last_was_an_operator = true;
//                     }
//                 },
//                 Ok(list @ ValuePart::List(..)) => {
//                     self.value_stack.push(list);
//                     last_was_an_operator = false;
//                 },
//                 Ok(color @ ValuePart::Color(..)) => {
//                     self.value_stack.push(color);
//                     last_was_an_operator = false;
//                 },
//                 Ok(ValuePart::Function(sass_function_call)) => {
//                     self.value_stack.push(try!(sass_function_call.evaluate(&variables)));
//                     last_was_an_operator = false;
//                 },
//                 Err(e) => return Err(e),
//             }
//         }
//
//         debug!("evaluate value_tokens empty: value_stack: {:?} op_stack: {:?}", self.value_stack, self.op_stack);
//
//         while !self.op_stack.is_empty() {
//             try!(self.math_machine());
//         }
//
//         if self.value_stack.len() > 1 {
//             // TODO: figure out how to make borrowck like not cloning this, it's cool i swear
//             Ok(ValuePart::List(self.value_stack.clone()))
//         } else {
//             self.pop_value()
//         }
//     }
//
//     fn push_on_list_on_value_stack(&mut self, push_val: ValuePart) -> Result<()> {
//         let list_starter = self.value_stack.pop().unwrap_or(ValuePart::List(vec![]));
//         debug!("evaluate push_on_list_on_value_stack list_starter: {:?} push_val: {:?}", list_starter, push_val);
//         let resulting_value = try!(ValuePart::concat_into_list(list_starter, push_val));
//         self.value_stack.push(resulting_value);
//         Ok(())
//     }
//
//     fn pop_value(&mut self) -> Result<ValuePart> {
//         self.value_stack.pop().ok_or(SassError {
//             offset: 0,
//             kind: ErrorKind::ExpectedValue,
//             message: String::from("Expected value to be on value stack but it was empty"),
//         })
//     }
//
//     fn pop_operator(&mut self) -> Result<Op> {
//         self.op_stack.pop().ok_or(SassError {
//             offset: 0,
//             kind: ErrorKind::ExpectedOperator,
//             message: String::from("Expected operator to be on operator stack but it was empty"),
//         })
//     }
//
//     fn math_machine(&mut self) -> Result<()> {
//         let op     = try!(self.pop_operator());
//         let second = try!(self.pop_value());
//         let first  = try!(self.pop_value());
//
//         debug!("evaluate math_machine first: {:?} op: {:?} second: {:?}", first, op, second);
//
//         let resulting_value = try!(op.apply(first, second, self.paren_level));
//         self.value_stack.push(resulting_value);
//         Ok(())
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use sass::value_part::ValuePart;
//     use sass::number_value::NumberValue;
//     use sass::op::Op;
//     use std::collections::HashMap;
//
//     #[test]
//     fn it_subtitutes_variable_values() {
//         let mut vars = HashMap::new();
//         vars.insert("$bar".into(), ValuePart::Number(NumberValue::from_scalar(4.0)));
//         vars.insert("$quux".into(), ValuePart::List(vec![
//             ValuePart::Number(NumberValue::with_units(3.0, String::from("px"))),
//             ValuePart::Number(NumberValue::with_units(10.0, String::from("px"))),
//         ]));
//
//         let answer = Evaluator::new_from_string(
//             "foo $bar 199.82 baz $quux"
//         ).evaluate(&vars);
//
//         assert_eq!(
//             Ok(ValuePart::List(vec![
//                 ValuePart::String("foo".into()),
//                 ValuePart::Number(NumberValue::computed(4.0)),
//                 ValuePart::Number(NumberValue::from_scalar(199.82)),
//                 ValuePart::String("baz".into()),
//                 ValuePart::Number(NumberValue::with_units(3.0, String::from("px"))),
//                 ValuePart::Number(NumberValue::with_units(10.0, String::from("px"))),
//             ])),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_flattents_lists() {
//         let answer = Evaluator::new_from_string(
//             "80% 90%, 80% 90%, 80% 90%"
//         ).evaluate(&HashMap::new());
//         assert_eq!(
//             Ok(ValuePart::List(vec![
//                 ValuePart::Number(NumberValue { scalar: 80.0, unit: Some(String::from("%")), computed: false}),
//                 ValuePart::Number(NumberValue { scalar: 90.0, unit: Some(String::from("%")), computed: false}),
//                 ValuePart::Operator(Op::Comma),
//                 ValuePart::Number(NumberValue { scalar: 80.0, unit: Some(String::from("%")), computed: false}),
//                 ValuePart::Number(NumberValue { scalar: 90.0, unit: Some(String::from("%")), computed: false}),
//                 ValuePart::Operator(Op::Comma),
//                 ValuePart::Number(NumberValue { scalar: 80.0, unit: Some(String::from("%")), computed: false}),
//                 ValuePart::Number(NumberValue { scalar: 90.0, unit: Some(String::from("%")), computed: false}),
//             ])),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_divides_if_value_came_from_a_variable() {
//         let mut vars = HashMap::new();
//         vars.insert("$three".into(), ValuePart::Number(NumberValue::computed(3.0)));
//
//         let answer = Evaluator::new_from_string("15 / $three").evaluate(&vars);
//
//         assert_eq!(
//             Ok(ValuePart::Number(NumberValue::computed(5.0))),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_divides_if_a_later_value_came_from_a_variable() {
//         let mut vars = HashMap::new();
//         vars.insert("$three".into(), ValuePart::Number(NumberValue::computed(3.0)));
//
//         let answer = Evaluator::new_from_string("15 / 5 / $three").evaluate(&vars);
//
//         assert_eq!(
//             Ok(ValuePart::Number(NumberValue::computed(1.0))),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_divides_out_units() {
//         let mut vars = HashMap::new();
//         vars.insert(
//             "$three".into(),
//             ValuePart::Number(NumberValue::with_units(3.0, String::from("px")))
//         );
//
//         let answer = Evaluator::new_from_string("15px / $three").evaluate(&vars);
//
//         assert_eq!(
//             Ok(ValuePart::Number(NumberValue::computed(5.0))),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_concats_colors_and_literals() {
//         let answer = Evaluator::new_from_string("#abc + hello").evaluate(&HashMap::new());
//
//         assert_eq!(
//             Ok(ValuePart::String("#abchello".into())),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_does_jacked_stuff() {
//         let mut vars = HashMap::new();
//         vars.insert("$stuff".into(), ValuePart::List(vec![
//             ValuePart::Number(NumberValue::computed(1.0)),
//             ValuePart::Number(NumberValue::computed(2.0)),
//             ValuePart::Number(NumberValue::computed(3.0)),
//         ]));
//         let answer = Evaluator::new_from_string(
//             "1/2, $stuff url(\"www.foo.com/blah.png\") blah blah"
//         ).evaluate(&vars);
//
//         assert_eq!(
//             Ok(ValuePart::List(vec![
//                 ValuePart::Number(NumberValue::from_scalar(1.0)),
//                 ValuePart::Operator(Op::Slash),
//                 ValuePart::Number(NumberValue::from_scalar(2.0)),
//                 ValuePart::Operator(Op::Comma),
//                 ValuePart::Number(NumberValue::computed(1.0)),
//                 ValuePart::Number(NumberValue::computed(2.0)),
//                 ValuePart::Number(NumberValue::computed(3.0)),
//                 ValuePart::String("url(\"www.foo.com/blah.png\")".into()),
//                 ValuePart::String("blah".into()),
//                 ValuePart::String("blah".into()),
//             ])),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_handles_lots_of_parens_and_slashes() {
//         let answer = Evaluator::new_from_string(
//             "1 + (2 + (3/4 + (4/5 6/7)))"
//         ).evaluate(&HashMap::new());
//         assert_eq!(
//             Ok(ValuePart::List(vec![
//                 ValuePart::String("120.750.8".into()),
//                 ValuePart::Number(NumberValue::from_scalar(6.0)),
//                 ValuePart::Operator(Op::Slash),
//                 ValuePart::Number(NumberValue::from_scalar(7.0)),
//             ])),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_handles_a_few_parens_and_slashes() {
//         let answer = Evaluator::new_from_string("(4/5 6/7)").evaluate(&HashMap::new());
//         assert_eq!(
//             Ok(ValuePart::List(vec![
//               ValuePart::Number(NumberValue::computed(0.8)),
//               ValuePart::Number(NumberValue::from_scalar(6.0)),
//               ValuePart::Operator(Op::Slash),
//               ValuePart::Number(NumberValue::from_scalar(7.0)),
//             ])),
//             answer
//         );
//     }
//
//     // Not sure what the correct behavior is here yet.
//     // #[test]
//     // fn it_handles_variables_and_parens() {
//     //     let mut vars = HashMap::new();
//     //     vars.insert("$foo".to_string(), ValuePart::List(vec![
//     //         ValuePart::List(vec![
//     //             ValuePart::Number(NumberValue::computed(4.0)),
//     //             ValuePart::Operator(Op::Comma),
//     //             ValuePart::Number(NumberValue::computed(5.0)),
//     //         ]),
//     //         ValuePart::Operator(Op::Comma),
//     //         ValuePart::Number(NumberValue::computed(6.0)),
//     //     ]));
//     //     let answer = Evaluator::new_from_string("3 + $foo").evaluate(&vars);
//     //
//     //     assert_eq!(
//     //         Ok(ValuePart::List(vec![
//     //             ValuePart::String(String::from("34")),
//     //             ValuePart::Operator(Op::Comma),
//     //             ValuePart::Number(NumberValue::computed(5.0)),
//     //             ValuePart::Operator(Op::Comma),
//     //             ValuePart::Number(NumberValue::computed(6.0)),
//     //         ])),
//     //         answer
//     //     );
//     // }
//
//     #[test]
//     fn it_adds() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::from_scalar(1.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(2.0))),
//         ]).evaluate(&HashMap::new());
//         assert_eq!(Ok(ValuePart::Number(NumberValue::computed(3.0))), answer);
//     }
//
//     #[test]
//     fn it_adds_with_units() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::with_units(1.0, String::from("px")))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Number(NumberValue::with_units(2.0, String::from("px")))),
//         ]).evaluate(&HashMap::new());
//         assert_eq!(
//             Ok(ValuePart::Number(
//                 NumberValue { scalar: 3.0, unit: Some(String::from("px")), computed: true }
//             )),
//             answer
//         );
//     }
//
//     #[test]
//     fn it_divides_and_adds_with_the_right_precedence() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(4.0))),
//         ]).evaluate(&HashMap::new());
//         assert_eq!(Ok(ValuePart::Number(NumberValue::computed(3.75))), answer);
//     }
//
//     #[test]
//     fn it_gets_the_right_precedence_with_parens() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Operator(Op::LeftParen)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Operator(Op::RightParen)),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(4.0))),
//         ]).evaluate(&HashMap::new());
//         assert_eq!(Ok(ValuePart::Number(NumberValue::computed(1.5))), answer);
//     }
//
//     #[test]
//     fn it_does_string_concat_when_adding_to_list() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::from_scalar(2.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Operator(Op::LeftParen)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Number(NumberValue::from_scalar(4.0))),
//             Ok(ValuePart::Operator(Op::RightParen)),
//         ]).evaluate(&HashMap::new());
//
//         assert_eq!(Ok(ValuePart::List(vec![
//             ValuePart::String("23".into()),
//             ValuePart::Number(NumberValue::from_scalar(4.0))
//         ])), answer);
//     }
//
//     #[test]
//     fn it_does_string_concat_when_adding_to_list_in_a_list() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Operator(Op::LeftParen)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(2.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Operator(Op::LeftParen)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Number(NumberValue::from_scalar(4.0))),
//             Ok(ValuePart::Operator(Op::RightParen)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(5.0))),
//             Ok(ValuePart::Operator(Op::RightParen)),
//         ]).evaluate(&HashMap::new());
//
//         assert_eq!(Ok(ValuePart::List(vec![
//             ValuePart::String("23".into()),
//             ValuePart::Number(NumberValue::from_scalar(4.0)),
//             ValuePart::Number(NumberValue::from_scalar(5.0)),
//         ])), answer);
//     }
//
//     #[test]
//     fn it_divides_because_parens_and_string_concats_because_list() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::from_scalar(1.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Operator(Op::LeftParen)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(5.0))),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(10.0))),
//             Ok(ValuePart::Number(NumberValue::from_scalar(2.0))),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Operator(Op::RightParen)),
//         ]).evaluate(&HashMap::new());
//
//         assert_eq!(Ok(ValuePart::List(vec![
//             ValuePart::String("10.5".into()),
//             ValuePart::Number(NumberValue::from_scalar(2.0)),
//             ValuePart::Number(NumberValue::from_scalar(3.0)),
//         ])), answer);
//     }
//
//     #[test]
//     fn it_does_not_divide_when_slash_is_separating() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::from_scalar(15.0))),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(3.0))),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(5.0))),
//         ]).evaluate(&HashMap::new());
//
//         assert_eq!(Ok(ValuePart::List(vec![
//             ValuePart::Number(NumberValue::from_scalar(15.0)),
//             ValuePart::Operator(Op::Slash),
//             ValuePart::Number(NumberValue::from_scalar(3.0)),
//             ValuePart::Operator(Op::Slash),
//             ValuePart::Number(NumberValue::from_scalar(5.0)),
//         ])), answer);
//     }
//
//     #[test]
//     fn it_does_divide_when_other_math_is_involved() {
//         let answer = Evaluator::new(vec![
//             Ok(ValuePart::Number(NumberValue::from_scalar(1.0))),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(2.0))),
//             Ok(ValuePart::Operator(Op::Plus)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(1.0))),
//             Ok(ValuePart::Operator(Op::Slash)),
//             Ok(ValuePart::Number(NumberValue::from_scalar(2.0))),
//         ]).evaluate(&HashMap::new());
//
//         assert_eq!(Ok(ValuePart::Number(NumberValue::computed(1.0))), answer);
//     }
// }
