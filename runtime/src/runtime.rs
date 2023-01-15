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
    for value in self.state.previous_events.values_mut() {
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

          // Add new frequency penalty for current event
          self.state.previous_events.insert(
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
      if !self.state.disabled_events.contains(&event.id)
        && (!self.state.previous_events.contains_key(&event.id)
          || self.state.previous_events[&event.id] >= 0)
      {
        result.push(event.id.clone());
      }
    }
    result
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
  use rand::SeedableRng;
  use rand_pcg::Pcg32;

  #[test]
  fn new_runtime_accepts_database() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database)
  }

  #[test]
  fn gets_random_event() {
    let db = load_mp_fixture("database").unwrap();
    let database = Database::from_u8(&db).unwrap();
    let mut runtime = Runtime::new(database.clone());
    let mut rng = Pcg32::seed_from_u64(1);
    let event = runtime.random_event(&mut rng).unwrap();
    assert_eq!(event, "choices");
  }

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
    assert!(runtime.state.previous_events.contains_key("event-1"));
    runtime.random_event(&mut rng).unwrap();
    assert!(runtime.state.previous_events.contains_key("event-2"));
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
}
