use rand::Rng;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait Probability: Debug {
  fn get_chance(&self) -> f32;
  fn roll_chance(&self) -> bool;
  fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PercentageProbability {
  pub value: u8,
}

#[typetag::serde]
impl Probability for PercentageProbability {
  fn get_chance(&self) -> f32 {
    self.value as f32 / 100.
  }

  fn roll_chance(&self) -> bool {
    rand::thread_rng().gen::<f32>() < self.get_chance()
  }
  fn as_any(&self) -> &dyn Any {
    self
  }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct FloatProbability {
  pub value: f32,
}

#[typetag::serde]
impl Probability for FloatProbability {
  fn get_chance(&self) -> f32 {
    self.value
  }

  fn roll_chance(&self) -> bool {
    rand::thread_rng().gen::<f32>() < self.get_chance()
  }
  fn as_any(&self) -> &dyn Any {
    self
  }
}
