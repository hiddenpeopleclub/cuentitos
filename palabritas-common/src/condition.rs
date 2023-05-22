use crate::Variable;

#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub enum Operator {
  #[default]
  Equal,
  NotEqual,
  GreaterThan,
  LessThan,
  GreaterOrEqualThan,
  LessOrEqualThan,
}

#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub struct Condition {
  pub variable: Variable,
  pub operator: Operator,
  pub value: String,
}

impl Condition {
  pub fn meets_condition(&self) -> bool {
    true
  }
}
