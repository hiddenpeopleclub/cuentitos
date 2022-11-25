use crate::*;
use crate::test_utils::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Event {
  id: String,
  unique: bool,
  title: String,
  description: String,
  choices: Vec<EventChoice>,
  requirements: Vec<EventRequirement>
}

#[derive(Default)]
pub struct EventBuilder {
  event: Event
}

impl EventBuilder {
  pub fn new() -> EventBuilder {
    EventBuilder {
      ..Default::default()
    }
  }

  pub fn id<T>(&mut self, id: T) -> &mut EventBuilder 
    where T: AsRef<str>  
  {
    self.event.id = id.as_ref().to_string();
    self
  }

  pub fn unique(&mut self) -> &mut EventBuilder {
    self.event.unique = true;
    self
  }

  pub fn title<T>(&mut self, title: T) -> &mut EventBuilder 
    where T: AsRef<str>  
  {
    self.event.title = title.as_ref().to_string();
    self
  }

  pub fn description<T>(&mut self, description: T) -> &mut EventBuilder 
    where T: AsRef<str>  
  {
    self.event.description = description.as_ref().to_string();
    self
  }

  pub fn choice(&mut self, choice: EventChoice) -> &mut EventBuilder {
    self.event.choices.push(choice);
    self
  }

  pub fn require(&mut self, requirement: EventRequirement) -> &mut EventBuilder {
    self.event.requirements.push(requirement);
    self
  }


  pub fn build(&mut self) -> Event {
    self.event.clone()
  }
}

#[cfg(test)]
mod test {
  use crate::{event::*, test_utils::load_mp_fixture};

  #[test]
  fn event_builder_supports_id() {
    let event = EventBuilder::new().build();
    assert_eq!(event.id, "");

    let event = EventBuilder::new()
                  .id("my-event")
                  .build();

    assert_eq!(event.id, "my-event");
  }

  #[test]
  fn event_builder_supports_unique() {
      let event = EventBuilder::new().build();
      assert_eq!(event.unique, false);

      let event = EventBuilder::new()
                    .unique()
                    .build();

      assert_eq!(event.unique, true);
  }

  #[test]
  fn event_builder_supports_title() {
    let event = EventBuilder::new().build();
    assert_eq!(event.title, "");

    let event = EventBuilder::new()
                  .title("My Event")
                  .build();

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

    let event = EventBuilder::new()
                  .choice(EventChoice::default())
                  .build();

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
