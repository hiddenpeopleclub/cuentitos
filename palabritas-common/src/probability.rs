use rand::Rng;

pub trait Probability {
  fn get_chance(&self) -> f32;
  fn roll_chance(&self) -> bool;
}

pub struct Percentage{
  value: u8,
}

impl Probability for Percentage
{
  fn get_chance(&self) -> f32{
    self.value as f32 /100.
  }

  fn roll_chance(&self) -> bool {
    rand::thread_rng().gen::<f32>() < self.get_chance()
  }
}

pub struct FloatPercentage{
  value: f32,
}


impl Probability for FloatPercentage
{
  fn get_chance(&self) -> f32{
    self.value
  }

  fn roll_chance(&self) -> bool {
    rand::thread_rng().gen::<f32>() < self.get_chance()
  }
}