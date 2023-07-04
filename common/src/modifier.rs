use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::VariableId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Modifier {
  pub variable: VariableId,
  pub value: String,
  pub operator: ModifierOperator,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ModifierOperator {
  #[default]
  Set,
  Add,
  Substract,
  Multiply,
  Divide,
}

impl Display for ModifierOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModifierOperator::Set => write!(f, "="),
      ModifierOperator::Add => write!(f, "+"),
      ModifierOperator::Substract => write!(f, "-"),
      ModifierOperator::Multiply => write!(f, "*"),
      ModifierOperator::Divide => write!(f, "/"),
    }
  }
}


impl Default for Modifier {
  fn default() -> Self {
    Self {
      variable: Default::default(),
      operator: Default::default(),
      value: "true".to_string(),
    }
  }
}