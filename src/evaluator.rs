use sass::value_part::ValuePart;
use value_tokenizer::ValueTokenizer;

use std::borrow::Cow::Borrowed;
use std::collections::HashMap;

pub fn evaluate(original: &str, variables: &HashMap<String, String>) -> String {
    let mut vt = ValueTokenizer::new(original);
    let mut value_stack = Vec::new();
    let mut op_stack = Vec::new();

    while let Some(part) = vt.next() {
        match part {
            ValuePart::Variable(name) => {
                match (*variables).get(&(*name).to_string()) {
                    Some(v) => value_stack.push(ValuePart::String(Borrowed(v))),
                    None => value_stack.push(ValuePart::String(name)),
                }
            },
            s @ ValuePart::String(..) => value_stack.push(s),
            n @ ValuePart::Number(..) => value_stack.push(n),
            ValuePart::Operator(ref o) => {
                while let Some(ValuePart::Operator(last_operator)) = op_stack.pop() {
                    if last_operator.same_or_greater_precedence(*o) {
                        let second = value_stack.pop().unwrap();
                        let first  = value_stack.pop().unwrap();
                        value_stack.push(last_operator.apply(first, second));
                    } else {
                        op_stack.push(ValuePart::Operator(last_operator));
                        break;
                    }
                }
                op_stack.push(ValuePart::Operator(*o));
            },
        }
    }

    while let Some(ValuePart::Operator(current_op)) = op_stack.pop() {
        let second = value_stack.pop().unwrap();
        let first  = value_stack.pop().unwrap();
        value_stack.push(current_op.apply(first, second));
    }

    value_stack.into_iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
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

        let answer = evaluate("foo $bar 199.82 baz $quux", &vars);
        assert_eq!("foo 4 199.82 baz 3px 10px", answer);
    }

    #[test]
    fn it_adds() {
        let answer = evaluate("1 + 2", &HashMap::new());
        assert_eq!("3", answer);
    }

    #[test]
    fn it_doesnt_need_space_around_operators() {
        let answer = evaluate("12*4", &HashMap::new());
        assert_eq!("48", answer);
    }

    #[test]
    fn it_divides_and_adds_with_the_right_precedence() {
        let answer = evaluate("3 + 3/4", &HashMap::new());
        assert_eq!("3.75", answer);
    }

    // #[test]
    // fn it_does_string_concat_when_adding_to_list() {
    //     let answer = evaluate("2+(3 4)", &HashMap::new());
    //     assert_eq!("23 4", answer);
    // }

    // #[test]
    // fn it_divides_because_parens_and_string_concats_because_list() {
    //     let answer = evaluate("1 + (5/10 2 3)", &HashMap::new());
    //     assert_eq!("10.5 2 3", answer);
    // }
}