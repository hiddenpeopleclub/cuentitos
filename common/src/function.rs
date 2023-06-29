use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Function {
  pub name: String,
  pub parameters: Vec<String>,
}
