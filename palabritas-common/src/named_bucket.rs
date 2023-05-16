use crate::{Content, Command, Probability};

#[derive(Default)]
pub struct NamedBucket{
  pub commands: Vec<Box<dyn Command>>,
  pub content: Vec<Box<dyn Content>>,
  pub probability: Option<Box<dyn Probability>>,
  pub name: String,
}

impl Content for NamedBucket{

}

