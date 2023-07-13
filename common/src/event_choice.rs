use crate::EventRequirement;
use crate::EventResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EventChoice {
  pub text: String,
  pub requirements: Vec<EventRequirement>,
  pub results: Vec<EventResult>,
}
