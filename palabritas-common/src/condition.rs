use crate::Variable;

#[derive(Default)]
pub enum Operator {
  #[default]
  Equal,
  NotEqual,
  GreaterThan,
  LessThan,
  GreaterOrEqualThan,
  LessOrEqualThan,
}

#[derive(Default)]
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
