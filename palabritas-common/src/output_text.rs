use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct OutputText {
  pub text: String,
  pub choices: Vec<String>,
}
