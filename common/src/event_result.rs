use crate::Modifier;
use serde::{ Deserialize, Serialize };
use crate::EventRequirement;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EventResult {
  pub chance: u8,
  pub text: String,
  pub requirements: Vec<EventRequirement>,
  pub modifiers: Vec<Modifier>
}
