use serde::{Deserialize, Serialize};

use crate::VariableId;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Modifier {
  pub variable: VariableId,
  pub added_value: String,
  pub is_override: bool,
}
