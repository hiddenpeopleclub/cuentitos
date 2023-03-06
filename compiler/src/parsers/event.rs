use crate::parsable::Parsable;
use crate::Config;
use cuentitos_common::EventRequirement;
use cuentitos_common::Modifier;
use regex::Regex;
use std::str::FromStr;

use cuentitos_common::EventBuilder;
use cuentitos_common::EventChoice;
use cuentitos_common::EventResult;

#[derive(Default)]
pub struct Event;

impl Parsable<cuentitos_common::Event> for Event {
  fn parse<S>(content: S, config: &Config) -> Result<cuentitos_common::Event, String>
  where
    S: AsRef<str>,
  {
    let mut builder = EventBuilder::new();
    let mut lines = content.as_ref().lines();

    builder.title(lines.next().unwrap());
    builder.description(lines.next().unwrap());

    let mut in_choice = false;
    let mut current_choice: EventChoice = EventChoice::default();

    let mut in_result = false;
    let mut current_result: EventResult = EventResult::default();

    let mut line_number = 2;

    for line in lines {
      if let Some((key, value)) = parse_setting(line) {
        if config.settings.contains(&key) {
          builder.set(key, value);
        } else {
          return Err(format!("Unknown setting '{}'", key));
        }
      }

      if let Some(choice) = parse_choice(line) {
        // We found a new choice, we add the current one...
        if in_choice {
          // If we were inside a result, we need to add it to the choice
          if in_result {
            current_choice.results.push(current_result);
            current_result = EventResult::default();
          }
          builder.choice(current_choice);
        }

        // ... and set the new one as current
        current_choice = choice;
        in_choice = true;
        in_result = false;
      }

      if let Some(result) = parse_result(line) {
        // We found a new result, we add the current one...
        if in_choice && in_result {
          current_choice.results.push(current_result);
        }

        current_result = result;
        in_result = true;
      }

      if let Some(requirement) = parse_requirement(line, config) {
        match requirement {
          Ok(requirement) => {
            if in_choice {
              if in_result {
                current_result.requirements.push(requirement);
              } else {
                current_choice.requirements.push(requirement);
              }
            } else {
              builder.require(requirement);
            }
          }
          Err(error) => return Err(format!("[{}] {}", line_number, error)),
        }
      }

      if let Some(modifier) = parse_modifier(line, config) {
        match modifier {
          Ok(modifier) => {
            if in_choice && in_result {
              current_result.modifiers.push(modifier);
            } else {
              return Err(format!(
                "[{}] found modifier outside a result: {}",
                line_number, line
              ));
            }
          }
          Err(error) => return Err(format!("[{}] {}", line_number, error)),
        }
      }
      line_number += 1;
    }

    // If we are done and we have an active choice and result, we add them.
    if in_choice {
      if in_result {
        current_choice.results.push(current_result)
      }
      builder.choice(current_choice);
    }

    Ok(builder.build())
  }
}

fn parse_setting(line: &str) -> Option<(String, String)> {
  let regexp = Regex::new(r"^set (.+) (.+)").unwrap();

  if let Some(result) = regexp.captures_iter(line).next() {
    return Some((result[1].to_string(), result[2].to_string()));
  }
  None
}

fn parse_choice(line: &str) -> Option<EventChoice> {
  let regexp = Regex::new(r"^\s+\* (.+)").unwrap();

  if let Some(result) = regexp.captures_iter(line).next() {
    return Some(EventChoice {
      text: result[1].to_string(),
      ..Default::default()
    });
  }
  None
}

fn parse_result(line: &str) -> Option<EventResult> {
  let regexp = Regex::new(r"^\s+\((\d+)\) (.+)").unwrap();

  if let Some(result) = regexp.captures_iter(line).next() {
    return Some(EventResult {
      chance: u32::from_str(&result[1]).unwrap(),
      text: result[2].to_string(),
      ..Default::default()
    });
  }
  None
}

fn parse_requirement(
  line: &str,
  config: &Config,
) -> Option<core::result::Result<EventRequirement, String>> {
  let regexp = Regex::new(r"^\s+require (.+)").unwrap();

  if let Some(result) = regexp.captures_iter(line).next() {
    return Some(crate::parsers::EventRequirement::parse(&result[1], config));
  }
  None
}

fn parse_modifier(line: &str, config: &Config) -> Option<core::result::Result<Modifier, String>> {
  let regexp = Regex::new(r"^\s+mod (.+)").unwrap();

  if let Some(result) = regexp.captures_iter(line).next() {
    return Some(crate::parsers::Modifier::parse(&result[1], config));
  }

  None
}
#[cfg(test)]
mod test {
  use crate::parsable::Parsable;
  use crate::Config;

  use cuentitos_common::Condition;
  use cuentitos_common::EventChoice;
  use cuentitos_common::EventRequirement;
  use cuentitos_common::EventResult;
  use cuentitos_common::Modifier;
  use cuentitos_common::Variable;
  use cuentitos_common::VariableKind::*;
  use cuentitos_common::TimeOfDay;

  use crate::parsers::Event;

  #[test]
  fn parse_parses_title_and_description_from_event() {
    let event = include_str!("../../fixtures/events/01-basic.event");
    let event = Event::parse(event, &Config::default()).unwrap();
    assert_eq!(event.title, "A Basic Event");
    assert_eq!(
      event.description,
      "This event has no options, just title and description"
    );
  }

  #[test]
  fn parse_parses_choices() {
    let event = include_str!("../../fixtures/events/02-choices.event");
    let event = Event::parse(event, &Config::default()).unwrap();
    println!("{:?}", event);
    assert_eq!(
      event.choices[0],
      EventChoice {
        text: "An Option without requirements".to_string(),
        ..Default::default()
      }
    );

    assert_eq!(
      event.choices[1],
      EventChoice {
        text: "Another option without requirements".to_string(),
        ..Default::default()
      }
    );
  }

  #[test]
  fn parse_parses_results() {
    let event = include_str!("../../fixtures/events/03-results.event");
    let event = Event::parse(event, &Config::default()).unwrap();
    assert_eq!(
      event.choices[0],
      EventChoice {
        text: "An Option".to_string(),
        results: vec![
          EventResult {
            chance: 10,
            text: "One Result".to_string(),
            ..Default::default()
          },
          EventResult {
            chance: 90,
            text: "Another Result".to_string(),
            ..Default::default()
          }
        ],
        ..Default::default()
      }
    );

    assert_eq!(
      event.choices[1],
      EventChoice {
        text: "Another option".to_string(),
        results: vec![
          EventResult {
            chance: 90,
            text: "One more Result".to_string(),
            ..Default::default()
          },
          EventResult {
            chance: 10,
            text: "Yet another Result".to_string(),
            ..Default::default()
          }
        ],
        ..Default::default()
      }
    );
  }

  #[test]
  fn parse_parses_requirements() {
    let mut config = Config::default();
    config.variables.insert("health".to_string(), Integer);
    config.variables.insert("happy".to_string(), Bool);
    config.variables.insert("tiles".to_string(), Enum { values: vec!["forest".to_string()] });
    config.reputations.push("rep_1".to_string());
    let event = include_str!("../../fixtures/events/04-requirements.event");
    let event = Event::parse(event, &config).unwrap();

    assert_eq!(
      event.requirements,
      vec![
        EventRequirement::Variable {
          variable: Variable {
            id: "health".to_string(),
            kind: Integer
          },
          condition: Condition::LessThan,
          amount: 100.to_string()
        },
        EventRequirement::Item {
          id: "wooden_figure".to_string(),
          condition: Condition::Equals,
          amount: 1.to_string()
        },
        EventRequirement::Reputation {
          id: "rep_1".to_string(),
          condition: Condition::HigherThan,
          amount: 5.to_string()
        },
        EventRequirement::TimeOfDay {
          id: TimeOfDay::Night,
          condition: Condition::Equals
        },
        EventRequirement::Event {
          id: "choices".to_string(),
          condition: Condition::MutEx
        },
        EventRequirement::Decision {
          id: "a_decision".to_string(),
          condition: Condition::Depends
        },
        EventRequirement::Tile {
          id: "forest".to_string(),
          condition: Condition::Depends
        }
      ]
    );
  }

  #[test]
  fn parse_parses_modifiers() {
    let mut config = Config::default();
    config.variables.insert("health".to_string(), Integer);
    config.variables.insert("happy".to_string(), Bool);
    config.reputations.push("rep_1".to_string());
    config.reputations.push("rep_2".to_string());

    let event = include_str!("../../fixtures/events/05-modifiers.event");
    let event = Event::parse(event, &config).unwrap();

    assert_eq!(
      event.choices[0].results[0].modifiers,
      vec![
        Modifier::Item {
          id: "wooden_figure".to_string(),
          amount: 1.to_string()
        },
        Modifier::Item {
          id: "wooden_figure".to_string(),
          amount: (-1).to_string()
        },
        Modifier::Resource {
          id: "health".to_string(),
          amount: (-2).to_string()
        },
        Modifier::Resource {
          id: "health".to_string(),
          amount: 5.to_string()
        },
        Modifier::Resource {
          id: "happy".to_string(),
          amount: true.to_string()
        },
        Modifier::Resource {
          id: "happy".to_string(),
          amount: false.to_string()
        },
        Modifier::Reputation {
          id: "rep_1".to_string(),
          amount: 2.to_string()
        },
        Modifier::Reputation {
          id: "rep_2".to_string(),
          amount: (-2).to_string()
        },
        Modifier::Decision("a_decision".to_string()),
        Modifier::Achievement("an_achievement".to_string())
      ]
    );
  }

  #[test]
  fn parse_parses_settings() {
    let event = include_str!("../../fixtures/events/06-settings.event");

    let mut config = Config::default();

    let parsed_event = Event::parse(event, &config);
    assert_eq!(parsed_event, Err("Unknown setting 'character'".to_string()));

    config.settings.push("character".to_string());
    config.settings.push("character_voice".to_string());

    let parsed_event = Event::parse(event, &config).unwrap();

    assert_eq!(
      parsed_event.settings.get("character").unwrap(),
      &"laidaxai".to_string()
    );

    assert_eq!(
      parsed_event.settings.get("character_voice").unwrap(),
      &"strong".to_string()
    )
  }

  // #[test]
  // fn parse_adds_error_on_missing_item() { todo!() }
  // #[test]
  // fn parse_adds_error_on_missing_reputation() { todo!() }
  // #[test]
  // fn parse_adds_error_on_wrong_time_of_day() { todo!() }
  // #[test]
  // fn parse_adds_warning_on_missing_decision() { todo!() }
  // #[test]
  // fn parse_adds_error_on_missing_event() { todo!() }
  // #[test]
  // fn parse_adds_error_on_missing_tile() { todo!() }

  // #[test]
  // fn use_regexp()
  // {
  //   // Puedo usar regexp para detectar si estoy adentro de un event,
  //   // choice o result para parsear requirements.
  //   // No usar builder mejor.
  //   todo!();
  // }
}
