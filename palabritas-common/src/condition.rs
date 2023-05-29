use crate::Variable;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Operator {
  #[default]
  Equal,
  NotEqual,
  GreaterThan,
  LessThan,
  GreaterOrEqualThan,
  LessOrEqualThan,

}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
