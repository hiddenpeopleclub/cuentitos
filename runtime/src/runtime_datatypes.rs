use cuentitos_common::LanguageId;
use serde::{Deserialize, Serialize};

// These are stripped down versions of the structs that
// that are sent through the runtime to the game engine.

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct Event {
  pub title: String,
  pub description: String,
  pub choices: Vec<EventChoice>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct EventChoice {
  pub id: usize,
  pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct EventResult {
  pub text: String,
  pub modifiers: Vec<Modifier>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct Modifier {
  pub kind: String,
  pub id: String,
  pub amount: String,
}

impl Event {
  pub fn from_cuentitos(event: &cuentitos_common::Event, i18n: &cuentitos_common::I18n, locale: &LanguageId) -> crate::Event {
    let mut choices = vec![];

    for (id, choice) in event.choices.iter().enumerate() {
      choices.push(EventChoice::from_cuentitos(id, choice, i18n, locale))
    }

    crate::Event {
      title: i18n.get_translation(locale, &event.title),
      description: i18n.get_translation(locale, &event.description),
      choices,
    }
  }
}

impl EventChoice {
  pub fn from_cuentitos(id: usize, choice: &cuentitos_common::EventChoice, i18n: &cuentitos_common::I18n, locale: &LanguageId) -> crate::EventChoice {
    crate::EventChoice {
        id,
        text: i18n.get_translation(locale, &choice.text),
      }
  }
}

impl EventResult {
  pub fn from_cuentitos(result: &cuentitos_common::EventResult, i18n: &cuentitos_common::I18n, locale: &LanguageId, modifiers: Vec<Modifier>) -> EventResult {
    crate::EventResult {
      text: i18n.get_translation(locale, &result.text),
      modifiers
    }
  }
}
