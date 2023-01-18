use serde::{Deserialize, Serialize};

// These are stripped down versions of the structs that
// that are sent through the runtime to the game engine.

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
  pub title: String,
  pub description: String,
  pub choices: Vec<EventChoice>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EventChoice {
  pub text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EventResult {
  pub text: String,
  pub modifiers: Vec<Modifier>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Modifier {
  kind: String,
  id: String,
  amount: String
}
