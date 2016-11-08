use operator::Operator;

use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OperatorOffset {
    pub operator: Operator,
    pub offset: Option<usize>,
}

impl fmt::Display for OperatorOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.operator.fmt(f)
    }
}
