use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type EventId = String;

#[derive(Default, Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Event {
  pub id: EventId,
  pub unique: bool,
  pub title: String,
  pub description: String,
  pub choices: Vec<EventChoice>,
  pub requirements: Vec<EventRequirement>,
  pub settings: HashMap<String, String>,
}

#[derive(Default)]
pub struct EventBuilder {
  event: Event,
}

impl EventBuilder {
  pub fn new() -> EventBuilder {
    EventBuilder {
      ..Default::default()
    }
  }

  pub fn id<T>(&mut self, id: T) -> &mut EventBuilder
  where
    T: AsRef<str>,
  {
    self.event.id = id.as_ref().to_string();
    self
  }

  pub fn unique(&mut self) -> &mut EventBuilder {
    self.event.unique = true;
    self
  }

  pub fn title<T>(&mut self, title: T) -> &mut EventBuilder
  where
    T: AsRef<str>,
  {
    self.event.title = title.as_ref().to_string();
    self
  }

  pub fn description<T>(&mut self, description: T) -> &mut EventBuilder
  where
    T: AsRef<str>,
  {
    self.event.description = description.as_ref().to_string();
    self
  }

  pub fn choice(&mut self, choice: EventChoice) -> &mut EventBuilder {
    self.event.choices.push(choice);
    self
  }

  pub fn choice_count(&self) -> usize {
    self.event.choices.len()
  }

  pub fn result_count(&self, choice: usize) -> usize {
    self.event.choices[choice].results.len()
  }

  pub fn require(&mut self, requirement: EventRequirement) -> &mut EventBuilder {
    self.event.requirements.push(requirement);
    self
  }

  pub fn set<T, U>(&mut self, key: T, value: U) -> &mut EventBuilder
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    let key = key.as_ref().to_string();
    let value = value.as_ref().to_string();

    self.event.settings.insert(key, value);
    self
  }
  pub fn build(&mut self) -> Event {
    self.event.clone()
  }
}

#[cfg(test)]
mod test {
  use crate::event::*;
  use crate::test_utils::{load_mp_fixture, serialize};

  #[test]
  fn event_builder_supports_id() {
    let event = EventBuilder::new().build();
    assert_eq!(event.id, "");

    let event = EventBuilder::new().id("my-event").build();

    assert_eq!(event.id, "my-event");
  }

  #[test]
  fn event_builder_supports_unique() {
    let event = EventBuilder::new().build();
    assert_eq!(event.unique, false);

    let event = EventBuilder::new().unique().build();

    assert_eq!(event.unique, true);
  }

  #[test]
  fn event_builder_supports_title() {
    let event = EventBuilder::new().build();
    assert_eq!(event.title, "");

    let event = EventBuilder::new().title("My Event").build();

    assert_eq!(event.title, "My Event");
  }

  #[test]
  fn event_builder_supports_description() {
    let event = EventBuilder::new().build();
    assert_eq!(event.description, "");

    let event = EventBuilder::new()
      .description("My event's description")
      .build();

    assert_eq!(event.description, "My event's description");
  }

  #[test]
  fn event_builder_supports_adding_choices() {
    let event = EventBuilder::new().build();
    assert_eq!(event.choices.len(), 0);

    let event = EventBuilder::new().choice(EventChoice::default()).build();

    assert_eq!(event.choices[0], EventChoice::default());
  }

  #[test]
  fn event_builder_supports_adding_requirements() {
    let event = EventBuilder::new().build();
    assert_eq!(event.requirements.len(), 0);

    let event = EventBuilder::new()
      .require(EventRequirement::default())
      .build();

    assert_eq!(event.requirements[0], EventRequirement::default());
  }

  #[test]
  fn event_builder_supports_settings() {
    let event = EventBuilder::new().build();
    assert_eq!(event.settings.len(), 0);

    let event = EventBuilder::new().set("an-option", "the-value").build();

    assert_eq!(
      event.settings.get(&"an-option".to_string()).unwrap(),
      &"the-value".to_string()
    );
  }

  #[test]
  fn event_serializes_to_message_pack() {
    let event = EventBuilder::new()
      .id("my-event")
      .unique()
      .title("My Event")
      .description("My event description")
      .choice(EventChoice::default())
      .require(EventRequirement::default())
      .build();

    let result = serialize(event).unwrap();
    let expected = load_mp_fixture("events/simple").unwrap();

    assert_eq!(result, expected)
  }
}
