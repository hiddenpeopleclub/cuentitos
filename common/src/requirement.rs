use crate::Condition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Requirement {
  pub condition: Condition,
}

impl Requirement {
  pub fn meets_requirement(&self) -> bool {
    self.condition.meets_condition()
  }
}
