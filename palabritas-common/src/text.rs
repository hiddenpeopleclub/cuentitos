use crate::{Content, Command, Probability};

#[derive(Default)]
pub struct Text{
  pub commands: Vec<Box<dyn Command>>,
  pub content: Vec<Box<dyn Content>>,
  pub probability: Option<Box<dyn Probability>>,
  pub text: String,
}

impl Content for Text{
  
}