use crate::Condition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Frequency {
  pub condition: Condition,
  pub change_value: f32,
}

impl Frequency {
  pub fn change_frequency(&self) -> f32 {
    match self.condition.meets_condition() {
      true => self.change_value,
      false => 0.,
    }
  }
}
