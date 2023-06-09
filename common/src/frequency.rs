use crate::Condition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct FrequencyModifier {
  pub condition: Condition,
  pub value: f32,
}

impl FrequencyModifier {
  pub fn change_frequency(&self) -> f32 {
    match self.condition.meets_condition() {
      true => self.value,
      false => 0.,
    }
  }
}
