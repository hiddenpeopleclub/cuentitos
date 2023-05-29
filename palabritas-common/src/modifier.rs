use crate::Variable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Modifier {
  pub variable: Variable,
  pub new_value: String,
}
