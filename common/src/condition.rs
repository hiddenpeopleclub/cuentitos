use crate::VariableId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ComparisonOperator {
  #[default]
  Equal,
  NotEqual,
  GreaterThan,
  LessThan,
  GreaterOrEqualThan,
  LessOrEqualThan,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Condition {
  pub variable: VariableId,
  pub operator: ComparisonOperator,
  pub value: String,
}

impl Default for Condition {
  fn default() -> Self {
    Self {
      variable: Default::default(),
      operator: Default::default(),
      value: "true".to_string(),
    }
  }
}
