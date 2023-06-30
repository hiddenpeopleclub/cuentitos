use crate::Condition;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct FrequencyModifier {
  pub condition: Condition,
  pub value: i32,
}
