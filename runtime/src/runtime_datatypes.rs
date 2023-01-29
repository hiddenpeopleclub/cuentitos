use serde::{Deserialize, Serialize};

// These are stripped down versions of the structs that
// that are sent through the runtime to the game engine.

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Event {
  pub title: String,
  pub description: String,
  pub choices: Vec<EventChoice>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EventChoice {
  pub id: usize,
  pub text: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EventResult {
  pub text: String,
  pub modifiers: Vec<Modifier>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Modifier {
  pub kind: String,
  pub id: String,
  pub amount: String,
}

impl Event {
  pub fn from_cuentitos(event: &cuentitos_common::Event) -> crate::Event {
    let mut choices = vec![];

    for (id, choice) in event.choices.iter().enumerate() {
      choices.push(crate::EventChoice {
        id,
        text: choice.text.clone(),
      })
    }

    crate::Event {
      title: event.title.clone(),
      description: event.description.clone(),
      choices,
    }
  }
}
