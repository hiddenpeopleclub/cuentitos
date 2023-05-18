use crate::Condition;

pub struct Requirement {
  pub condition: Condition,
}

impl Requirement {
  pub fn meets_requirement(&self) -> bool {
    self.condition.meets_condition()
  }
}
