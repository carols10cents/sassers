use sass::value_part::ValuePart;

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

fn is_space(c: u8) -> bool {
    c == b' '
}

fn isnt_space(c: u8) -> bool {
    !is_space(c)
}

fn is_number(c: u8) -> bool {
    let result = match c {
        b'0' ... b'9' | b'.' => true,
        _ => false,
    };
    result
}

fn is_operator(c: u8) -> bool {
    match c {
        b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')' | b',' => true,
        _ => false,
    }
}

#[derive(Debug)]
pub struct ValueTokenizer<'a> {
    value_str: &'a str,
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ValueTokenizer<'a> {
    pub fn new(value_str: &'a str) -> ValueTokenizer<'a> {
        ValueTokenizer {
            value_str: &value_str,
            bytes: &value_str.as_bytes(),
            offset: 0,
        }
    }

    pub fn parse(&mut self) -> Option<ValuePart<'a>> {
        self.skip_leading_whitespace();

        let start = self.offset;
        let mut i = self.offset;
        let limit = self.value_str.len();

        if is_operator(self.bytes[start]) {
            self.offset = start + 1;
            Some(ValuePart::Operator(self.value_str[start..start + 1].parse().unwrap()))
        } else if is_number(self.bytes[start]) {
            i += self.scan_while(&self.value_str[i..limit], is_number);
            self.offset = i;
            Some(ValuePart::Number(self.value_str[start..i].parse().unwrap()))
        } else if self.bytes[start] == b'$' {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i;
            Some(ValuePart::Variable(Borrowed(&self.value_str[start..i])))
        } else {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i;
            Some(ValuePart::String(Borrowed(&self.value_str[start..i])))
        }
    }

    fn scan_while<F>(&mut self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
        }
    }

    fn skip_leading_whitespace(&mut self) {
       let mut i = self.offset;
       let limit = self.value_str.len();

       while i < limit {
           let c = self.bytes[i];
           if is_space(c) {
               i += self.scan_while(&self.value_str[i..limit], is_space);
           } else {
               self.offset = i;
               return
           }
       }
       self.offset = limit;
   }
}

impl<'a> Iterator for ValueTokenizer<'a> {
    type Item = ValuePart<'a>;

    fn next(&mut self) -> Option<ValuePart<'a>> {
        if self.offset < self.value_str.len() {
            return self.parse()
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::value_part::ValuePart;
    use sass::op::Op;
    use std::borrow::Cow::Borrowed;
    use std::collections::HashMap;

    #[test]
    fn it_returns_string_part() {
        let mut vt = ValueTokenizer::new("foo");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_space_separated_string_parts() {
        let mut vt = ValueTokenizer::new("foo bar");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(Some(ValuePart::String(Borrowed(&"bar"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variable() {
        let mut vt = ValueTokenizer::new("$foo");
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$foo"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variables_and_string_parts() {
        let mut vt = ValueTokenizer::new("foo $bar baz $quux");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$bar"))), vt.next());
        assert_eq!(Some(ValuePart::String(Borrowed(&"baz"))), vt.next());
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$quux"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_number() {
        let mut vt = ValueTokenizer::new("3");
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_two_numbers() {
        let mut vt = ValueTokenizer::new("3 8.9");
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(Some(ValuePart::Number(8.9)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_operator() {
        let mut vt = ValueTokenizer::new("+");
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_and_operators() {
        let mut vt = ValueTokenizer::new("6 + 75.2");
        assert_eq!(Some(ValuePart::Number(6.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(Some(ValuePart::Number(75.2)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_does_stuff_with_parens() {
        let mut vt = ValueTokenizer::new("2+(3 4)");
        assert_eq!(Some(ValuePart::Number(2.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::LeftParen)), vt.next());
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(Some(ValuePart::Number(4.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::RightParen)), vt.next());
        assert_eq!(None, vt.next());
    }

    // evaluate tests =====================

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
