use std::fmt::Display;

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

impl Display for ComparisonOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ComparisonOperator::Equal => write!(f, "="),
      ComparisonOperator::NotEqual => write!(f, "!"),
      ComparisonOperator::GreaterThan => write!(f, ">"),
      ComparisonOperator::LessThan => write!(f, "<"),
      ComparisonOperator::GreaterOrEqualThan => write!(f, ">="),
      ComparisonOperator::LessOrEqualThan => write!(f, "<="),
    }
  }
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
