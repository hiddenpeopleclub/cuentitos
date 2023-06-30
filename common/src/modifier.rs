use serde::{Deserialize, Serialize};

use crate::VariableId;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
