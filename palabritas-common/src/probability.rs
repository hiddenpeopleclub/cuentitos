use rand::Rng;
use std::any::Any;
use std::fmt::Debug;
pub trait Probability: Debug {
  fn get_chance(&self) -> f32;
  fn roll_chance(&self) -> bool;
  fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PercentageProbability {
  pub value: u8,
}

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

#[derive(Debug, Default, PartialEq, Clone)]
pub struct FloatProbability {
  pub value: f32,
}

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
