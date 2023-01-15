use cuentitos_common::Condition;
use cuentitos_common::Event;
use cuentitos_common::EventRequirement;
use cuentitos_common::ResourceKind;

use crate::GameState;
use crate::RuntimeState;
use cuentitos_common::Database;
use cuentitos_common::EventId;
use rand::Rng;
use rand_pcg::Pcg32;

use rmp_serde::Deserializer;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Runtime {
  database: Database,
  id_to_index: HashMap<EventId, usize>,
  state: RuntimeState,
  game_state: GameState,
}

impl Runtime {
  pub fn new(database: Database) -> Runtime {
    let mut runtime = Runtime {
      database,
      ..Default::default()
    };
    for index in 0..runtime.database.events.len() {
      let id = &runtime.database.events[index].id;
      runtime.id_to_index.insert(id.clone(), index);
    }
    runtime
  }

  pub fn random_event(&mut self, rng: &mut Pcg32) -> Option<EventId> {
    // Update previous event frequencies
    for value in self.state.previous_event_cooldown.values_mut() {
      *value += self.database.config.runtime.event_frequency_cooldown;
    }

    // Remove elements that don't meet current state requirements
    // Calculate frequency of each event given current state
    //  - Augment frequency if conditions met
    let frequencies = self.event_frequency_sum(self.event_frequencies(&GameState::default()));
    if let Some(max) = frequencies.last() {
      let num = rng.gen_range(0..*max);
      let mut index = 0;

      for freq in frequencies {
        if num <= freq {
          let event_id = self.available_events()[index].clone();
          let index = self.id_to_index[&event_id];
          let event = &self.database.events[index];

          // Disable event if unique
          if event.unique {
            self.state.disabled_events.push(event_id.clone())
          }

          // Add current event to previous events list
          if !self.state.previous_events.contains(&event_id) {
            self.state.previous_events.push(event_id.clone())
          };

          // Add new frequency penalty for current event
          self.state.previous_event_cooldown.insert(
            event.id.clone(),
            self.database.config.runtime.chosen_event_frequency_penalty,
          );

          return Some(event_id);
        } else {
          index += 1;
        };
      }
      None
    } else {
      None
    }

    // match self.available_events().choose(rng) {
    //   Some(event_id) => {
    //   }
    //   None => None,
    // }
  }

  fn available_events(&self) -> Vec<EventId> {
    let mut result = vec![];

    for event in &self.database.events {
      if !self.state.disabled_events.contains(&event.id) && // Event is not disabled (unique that was chosen)

        (!self.state.previous_event_cooldown.contains_key(&event.id) // Event has not been chosen before, or cooldown happened
          || self.state.previous_event_cooldown[&event.id] >= 0) &&     //

        self.resource_requirements_met(event)
      {
        result.push(event.id.clone());
      }
    }
    result
  }

  fn resource_requirements_met(&self, event: &Event) -> bool {
    for requirement in &event.requirements {
      match requirement {
        EventRequirement::Resource {
          resource,
          condition,
          amount,
        } => {
          let current_value = self.game_state.resources[&resource.id].clone();
          match resource.kind {
            ResourceKind::Integer => {
              let cv = current_value.parse::<i32>().unwrap_or(0);
              let a = amount.parse::<i32>().unwrap_or(0);
              match condition {
                Condition::Equals => return cv == a,
                Condition::HigherThan => return cv > a,
                Condition::LessThan => return cv < a,
                _ => {}
              }
            }
            ResourceKind::Float => {
              let cv = current_value.parse::<f32>().unwrap_or(0.0);
              let a = amount.parse::<f32>().unwrap_or(0.0);
              match condition {
                Condition::Equals => return cv == a,
                Condition::HigherThan => return cv > a,
                Condition::LessThan => return cv < a,
                _ => {}
              }
            }
            ResourceKind::Bool => {
              let cv = current_value.parse::<bool>().unwrap_or(false);
              let a = amount.parse::<bool>().unwrap_or(false);

              if condition == &Condition::Equals {
                return cv == a;
              }
            }
          }
        }
        EventRequirement::Item {
          id,
          condition,
          amount,
        } => {
          let cv = self
            .game_state
            .items
            .get(id)
            .unwrap_or(&"0".to_string())
            .parse::<i32>()
            .unwrap_or(0);
          let a = amount.parse::<i32>().unwrap_or(0);
          match condition {
            Condition::Equals => return cv == a,
            Condition::HigherThan => return cv > a,
            Condition::LessThan => return cv < a,
            _ => {}
          }
        }
        EventRequirement::Reputation {
          id,
          condition,
          amount,
        } => {
          let cv = self
            .game_state
            .reputations
            .get(id)
            .unwrap_or(&"0".to_string())
            .parse::<i32>()
            .unwrap_or(0);
          let a = amount.parse::<i32>().unwrap_or(0);
          match condition {
            Condition::Equals => return cv == a,
            Condition::HigherThan => return cv > a,
            Condition::LessThan => return cv < a,
            _ => return false,
          }
        }
        EventRequirement::TimeOfDay { id, condition } => {
          let cv = self.game_state.time_of_day.clone();
          match condition {
            Condition::Equals => return cv == *id,
            Condition::MutEx => return cv != *id,
            _ => return false,
          }
        }
        EventRequirement::Decision { id, condition } => match condition {
          Condition::Depends => return self.game_state.decisions.contains(id),
          Condition::MutEx => return !self.game_state.decisions.contains(id),
          _ => return false,
        },
        EventRequirement::Event { id, condition } => match condition {
          Condition::Depends => return self.state.previous_events.contains(id),
          Condition::MutEx => return !self.state.previous_events.contains(id),
          _ => return false,
        },
        EventRequirement::Tile { id, condition } => match condition {
          Condition::Equals => return self.game_state.tile == *id,
          Condition::MutEx => return self.game_state.tile != *id,
          _ => return false,
        }
        EventRequirement::Empty => {}
      }
    }
    true
  }

  fn event_frequencies(&self, _game_state: &GameState) -> Vec<i32> {
    let mut result = vec![];

    for _idx in self.available_events() {
      result.push(50);
    }

    result
  }

  fn event_frequency_sum(&self, frequencies: Vec<i32>) -> Vec<i32> {
    let mut result = vec![];

    for freq in frequencies {
      let prev = result.last().unwrap_or(&0);
      result.push(prev + freq)
    }

    result
  }

  pub fn save<T>(&self, path: T) -> cuentitos_common::Result<()>
  where
    T: AsRef<Path>,
  {
    let mut buf: Vec<u8> = Vec::new();
    let mut serializer = Serializer::new(&mut buf);
    self.serialize(&mut serializer)?;
    let mut file = File::create(path)?;
    file.write_all(&buf)?;
    Ok(())
  }

  pub fn load<T>(path: T) -> cuentitos_common::Result<Runtime>
  where
    T: AsRef<Path>,
  {
    let mut f = File::open(&path)?;
    let metadata = fs::metadata(&path)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer)?;
    let buf: &[u8] = &buffer;
    let mut de = Deserializer::new(buf);
    let runtime: std::result::Result<Runtime, rmp_serde::decode::Error> =
      Deserialize::deserialize(&mut de);
    match runtime {
      Ok(runtime) => Ok(runtime),
      Err(error) => Err(Box::new(error)),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::runtime::Runtime;
  use crate::GameState;
  use cuentitos_common::test_utils::load_mp_fixture;
  use cuentitos_common::Database;
  use cuentitos_common::Event;
  use cuentitos_common::*;
  use rand::SeedableRng;
  use rand_pcg::Pcg32;

  #[test]
  fn new_runtime_accepts_database() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database)
  }

  // #[test]
  // fn gets_random_event() {
  //   let db = load_mp_fixture("database").unwrap();
  //   let database = Database::from_u8(&db).unwrap();
  //   let mut runtime = Runtime::new(database.clone());
  //   let mut rng = Pcg32::seed_from_u64(1);
  //   let event = runtime.random_event(&mut rng).unwrap();
  //   assert_eq!(event, "choices");
  // }

  #[test]
  fn runtime_can_be_saved_and_loaded() {
    let mut path = std::env::temp_dir().to_path_buf();
    path.push("state_save.mp");

    let db = load_mp_fixture("database").unwrap();
    let database = Database::from_u8(&db).unwrap();
    let runtime = Runtime::new(database.clone());

    runtime.save(&path).unwrap();

    let runtime2 = Runtime::load(path).unwrap();

    assert_eq!(runtime, runtime2);
  }

  #[test]
  fn choosing_event_stores_it_in_previous_events() {
    let db = Database {
      events: vec![
        Event {
          id: "event-1".to_string(),
          ..Default::default()
        },
        Event {
          id: "event-2".to_string(),
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);
    let mut rng = Pcg32::seed_from_u64(1);

    runtime.random_event(&mut rng).unwrap();
    assert!(runtime
      .state
      .previous_event_cooldown
      .contains_key("event-1"));
    runtime.random_event(&mut rng).unwrap();
    assert!(runtime
      .state
      .previous_event_cooldown
      .contains_key("event-2"));
  }

  #[test]
  fn unique_events_show_up_once() {
    let db = Database {
      events: vec![
        Event {
          id: "event-1".to_string(),
          unique: true,
          ..Default::default()
        },
        Event {
          id: "event-2".to_string(),
          ..Default::default()
        },
      ],
      ..Default::default()
    };
    let mut runtime = Runtime::new(db);
    let mut rng = Pcg32::seed_from_u64(1);

    runtime.random_event(&mut rng).unwrap();
    assert_eq!(runtime.state.disabled_events, ["event-1"]);
    assert_eq!(runtime.available_events(), ["event-2"]);
  }

  #[test]
  fn available_events_ignores_previous_events_in_cooldown_mode() {
    let db = Database {
      events: vec![
        Event {
          id: "event-1".to_string(),
          ..Default::default()
        },
        Event {
          id: "event-2".to_string(),
          ..Default::default()
        },
      ],
      ..Default::default()
    };
    let mut runtime = Runtime::new(db);
    let mut rng = Pcg32::seed_from_u64(1);

    let game_state = GameState::default();

    assert_eq!(runtime.available_events(), ["event-1", "event-2"]);
    assert_eq!(runtime.event_frequencies(&game_state), [50, 50]);

    runtime.random_event(&mut rng).unwrap();
    assert_eq!(runtime.available_events(), ["event-2"]);
    assert_eq!(runtime.event_frequencies(&game_state), [50]);

    runtime.random_event(&mut rng).unwrap();
    assert!(runtime.available_events().is_empty());
    assert!(runtime.event_frequencies(&game_state).is_empty());
    assert_eq!(runtime.random_event(&mut rng), None);

    // Make sure that after a while it shows up again
    for _ in 0..7 {
      runtime.random_event(&mut rng);
    }

    let event = runtime.random_event(&mut rng).unwrap();
    assert_eq!(event, "event-1");
    assert!(runtime.available_events().is_empty());
    assert!(runtime.event_frequencies(&game_state).is_empty());
  }

  #[test]
  fn requirements_on_integer_resource_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-resource-integer-higher-than".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Integer,
            },
            condition: Condition::HigherThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-resource-integer-less-than".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Integer,
            },
            condition: Condition::LessThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-resource-integer-equals".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Integer,
            },
            condition: Condition::Equals,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let resource = "resource-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "2".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-integer-less-than"]
    );

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "12".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-integer-higher-than"]
    );

    runtime
      .game_state
      .resources
      .insert(resource, "10".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-integer-equals"]
    );
  }

  #[test]
  fn requirements_on_float_resource_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-resource-float-higher-than".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Float,
            },
            condition: Condition::HigherThan,
            amount: "10.5".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-resource-float-less-than".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Float,
            },
            condition: Condition::LessThan,
            amount: "10.5".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-resource-float-equals".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Float,
            },
            condition: Condition::Equals,
            amount: "10.5".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let resource = "resource-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "2.5".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-float-less-than"]
    );

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "12.5".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-float-higher-than"]
    );

    runtime
      .game_state
      .resources
      .insert(resource, "10.5".to_string());
    assert_eq!(runtime.available_events(), ["event-resource-float-equals"]);
  }

  #[test]
  fn requirements_on_bool_resource_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-resource-bool-equals-true".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Bool,
            },
            condition: Condition::Equals,
            amount: "true".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-resource-bool-equals-false".to_string(),
          requirements: vec![EventRequirement::Resource {
            resource: Resource {
              id: "resource-1".to_string(),
              kind: ResourceKind::Bool,
            },
            condition: Condition::Equals,
            amount: "false".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let resource = "resource-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "true".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-bool-equals-true"]
    );

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "false".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-bool-equals-false"]
    );
  }

  #[test]
  fn requirements_on_items_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-item-equals".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "item".to_string(),
            condition: Condition::Equals,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-higher-than".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "item".to_string(),
            condition: Condition::HigherThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-less-than".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "item".to_string(),
            condition: Condition::LessThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-missing-higher".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "missing".to_string(),
            condition: Condition::HigherThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-missing-less".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "missing".to_string(),
            condition: Condition::LessThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let item = "item".to_string();
    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .items
      .insert(item.clone(), "1".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-item-less-than", "event-item-missing-less"]
    );

    runtime
      .game_state
      .items
      .insert(item.clone(), "15".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-item-higher-than", "event-item-missing-less"]
    );

    runtime
      .game_state
      .items
      .insert(item.clone(), "10".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-item-equals", "event-item-missing-less"]
    );
  }

  #[test]
  fn requirements_on_reputations_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-reputation-equals".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "reputation".to_string(),
            condition: Condition::Equals,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-higher-than".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "reputation".to_string(),
            condition: Condition::HigherThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-less-than".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "reputation".to_string(),
            condition: Condition::LessThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-missing-higher".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "missing".to_string(),
            condition: Condition::HigherThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-missing-less".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "missing".to_string(),
            condition: Condition::LessThan,
            amount: "10".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let reputation = "reputation".to_string();
    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .reputations
      .insert(reputation.clone(), "1".to_string());
    assert_eq!(
      runtime.available_events(),
      [
        "event-reputation-less-than",
        "event-reputation-missing-less"
      ]
    );

    runtime
      .game_state
      .reputations
      .insert(reputation.clone(), "15".to_string());
    assert_eq!(
      runtime.available_events(),
      [
        "event-reputation-higher-than",
        "event-reputation-missing-less"
      ]
    );

    runtime
      .game_state
      .reputations
      .insert(reputation.clone(), "10".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-reputation-equals", "event-reputation-missing-less"]
    );
  }

  #[test]
  fn requirements_on_time_of_day_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-tod-morning".to_string(),
          requirements: vec![EventRequirement::TimeOfDay {
            id: TimeOfDay::Morning,
            condition: Condition::Equals,
          }],
          ..Default::default()
        },
        Event {
          id: "event-tod-mutex-night".to_string(),
          requirements: vec![EventRequirement::TimeOfDay {
            id: TimeOfDay::Night,
            condition: Condition::MutEx,
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    runtime.game_state.time_of_day = TimeOfDay::Morning;
    assert_eq!(
      runtime.available_events(),
      ["event-tod-morning", "event-tod-mutex-night"]
    );

    runtime.game_state.time_of_day = TimeOfDay::Evening;
    assert_eq!(runtime.available_events(), ["event-tod-mutex-night"]);

    runtime.game_state.time_of_day = TimeOfDay::Night;
    assert!(runtime.available_events().is_empty());
  }

  #[test]
  fn requirements_on_decisions_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-decision".to_string(),
          requirements: vec![EventRequirement::Decision {
            id: "decision".to_string(),
            condition: Condition::Depends,
          }],
          ..Default::default()
        },
        Event {
          id: "event-decision-2".to_string(),
          requirements: vec![EventRequirement::Decision {
            id: "decision-2".to_string(),
            condition: Condition::MutEx,
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    runtime.game_state.decisions = vec![];
    assert_eq!(runtime.available_events(), ["event-decision-2"]);

    runtime.game_state.decisions = vec!["decision".to_string()];
    assert_eq!(
      runtime.available_events(),
      ["event-decision", "event-decision-2"]
    );

    runtime.game_state.decisions = vec!["decision".to_string(), "decision-2".to_string()];
    assert_eq!(runtime.available_events(), ["event-decision"]);
  }

  #[test]
  fn requirements_on_events_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-depends".to_string(),
          requirements: vec![EventRequirement::Event {
            id: "event-1".to_string(),
            condition: Condition::Depends,
          }],
          ..Default::default()
        },
        Event {
          id: "event-mutex".to_string(),
          requirements: vec![EventRequirement::Event {
            id: "event-2".to_string(),
            condition: Condition::MutEx,
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    runtime.state.previous_events = vec![];
    assert_eq!(runtime.available_events(), ["event-mutex"]);

    runtime.state.previous_events = vec!["event-1".to_string()];
    assert_eq!(runtime.available_events(), ["event-depends", "event-mutex"]);

    runtime.state.previous_events = vec!["event-1".to_string(), "event-2".to_string()];
    assert_eq!(runtime.available_events(), ["event-depends"]);
  }

  #[test]
  fn requirements_on_tile_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-tile-forest".to_string(),
          requirements: vec![EventRequirement::Tile {
            id: "forest".to_string(),
            condition: Condition::Equals,
          }],
          ..Default::default()
        },
        Event {
          id: "event-tile-not-forest".to_string(),
          requirements: vec![EventRequirement::Tile {
            id: "forest".to_string(),
            condition: Condition::MutEx,
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    runtime.game_state.tile = "plain".to_string();
    assert_eq!(
      runtime.available_events(),
      ["event-tile-not-forest"]
    );

    runtime.game_state.tile = "forest".to_string();
    assert_eq!(runtime.available_events(), ["event-tile-forest"]);
  }

}
