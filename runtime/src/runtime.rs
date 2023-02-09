use cuentitos_common::EventChoice;
use cuentitos_common::EventResult;
use cuentitos_common::Modifier;
use cuentitos_common::ReputationId;
use cuentitos_common::TileId;

use cuentitos_common::TimeOfDay;

use cuentitos_common::Condition;
use cuentitos_common::Event;
use cuentitos_common::EventRequirement;
use cuentitos_common::ResourceKind;
use rand::Rng;
use rand::SeedableRng;

use crate::GameState;
use crate::RuntimeState;

use cuentitos_common::Database;
use cuentitos_common::EventId;
use rand_pcg::Pcg32;

use rmp_serde::Deserializer;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Runtime {
  pub database: Database,
  id_to_index: HashMap<EventId, usize>,
  state: RuntimeState,
  game_state: GameState,
  #[serde(skip)]
  rng: Option<Pcg32>,
  seed: u64,
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

  pub fn get(&self, event_id: EventId) -> Option<&Event> {
    match self.id_to_index.get(&event_id) {
      Some(index) => self.database.events.get(*index),
      None => None,
    }
  }

  pub fn next_event(&mut self) -> Option<crate::Event> {
    // Update previous event frequencies
    for value in self.state.previous_event_cooldown.values_mut() {
      *value += self.database.config.runtime.event_frequency_cooldown as i32;
    }

    // Use frequency of each event given current state for the random selection
    let frequencies = frequency_sum(self.event_frequencies());

    if let Some(max) = frequencies.last() {
      if let Some(num) = random_with_max(self, *max) {
        let mut index = 0;

        for freq in frequencies {
          if num <= freq {
            let event_id = self.available_events()[index].clone();
            let index = self.id_to_index[&event_id];
            let event = &self.database.events[index];

            self.state.current_event = Some(index);

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

            return Some(crate::Event::from_cuentitos(event));
          } else {
            index += 1;
          };
        }
      }
      None
    } else {
      None
    }
  }

  pub fn set_seed(&mut self, seed: u64) {
    self.seed = seed;
    self.rng = Some(Pcg32::seed_from_u64(seed));
  }
  pub fn set_choice(&mut self, choice_id: usize) -> Result<crate::EventResult, String> {
    if let Some(event) = current_event(self) {
      if event.choices.len() > choice_id {
        let choice = &event.choices[choice_id];

        let mut available = true;

        for requirement in &choice.requirements {
          available = available && self.requirement_met(&requirement);
        }

        if available {
          self.state.current_choice = Some(choice_id);
         if let Some(result) = self.random_result()
         {
          if let Some(modifiers) = self.current_modifiers()
          {
            let event_result = crate::EventResult
            {
                text: result.text.clone(),
                modifiers,
            };
            return Ok(event_result);
          } else {
            return Err("Invalid modifiers".to_string());
          }
         } else {
          return Err("Invalid results".to_string());
         }
   
        } else {
          self.state.current_event = None;
          return Err("Requirements for choice not met".to_string());
        }
      } else {
        self.state.current_choice = None;
      }
      Err("Invalid choice".to_string())
    } else {
      self.state.current_event = None;
      Err("No event has been drawn".to_string())
    }
  }

  pub fn current_modifiers(&self) -> Option<Vec<crate::Modifier>> {
    let mut v = vec![];

    let result = current_result(self)?;
    for modifier in &result.modifiers {
      let modifier = modifier.clone();
      match modifier {
        Modifier::Resource { id, amount } => v.push(crate::Modifier {
          kind: "resource".to_string(),
          id,
          amount,
        }),
        Modifier::Item { id, amount } => v.push(crate::Modifier {
          kind: "item".to_string(),
          id,
          amount,
        }),
        Modifier::Reputation { id, amount } => v.push(crate::Modifier {
          kind: "reputation".to_string(),
          id,
          amount,
        }),
        Modifier::Decision(id) => v.push(crate::Modifier {
          kind: "decision".to_string(),
          id,
          amount: String::default(),
        }),
        Modifier::Achievement(id) => v.push(crate::Modifier {
          kind: "achievement".to_string(),
          id,
          amount: String::default(),
        }),
        _ => {}
      }
    }

    Some(v)
  }

  pub fn set_resource<R, T>(&mut self, resource: R, value: T) -> Result<(), String>
  where
    T: Display,
    R: AsRef<str>,
  {
    let resource = resource.as_ref().to_string();
    if self.database.config.resources.contains_key(&resource) {
      let t = std::any::type_name::<T>();
      if (t == "i32" && self.database.config.resources[&resource] == ResourceKind::Integer)
        || (t == "f32" && self.database.config.resources[&resource] == ResourceKind::Float)
        || (t == "bool" && self.database.config.resources[&resource] == ResourceKind::Bool)
      {
        self
          .game_state
          .resources
          .insert(resource, value.to_string());
      } else {
        return Err("Invalid Resource Type".to_string());
      }
    } else {
      return Err("Invalid Resource".to_string());
    }
    Ok(())
  }

  pub fn get_resource_kind<R>(&self, resource: R) -> Option<ResourceKind>
  where
    R: AsRef<str>,
  {
    let resource = resource.as_ref();

    if self.database.config.resources.contains_key(resource) {
      Some(self.database.config.resources[resource].clone())
    } else {
      None
    }
  }

  pub fn get_resource<R, T>(&self, resource: R) -> Result<T, String>
  where
    T: Display + std::str::FromStr + Default,
    R: AsRef<str>,
  {
    let resource = resource.as_ref().to_string();
    if self.database.config.resources.contains_key(&resource) {
      let t = std::any::type_name::<T>();

      if (t == "i32" && self.database.config.resources[&resource] == ResourceKind::Integer)
        || (t == "f32" && self.database.config.resources[&resource] == ResourceKind::Float)
        || (t == "bool" && self.database.config.resources[&resource] == ResourceKind::Bool)
      {
        let value = match self.game_state.resources.get(&resource) {
          Some(value) => value.clone(),
          None => T::default().to_string(),
        };

        if let Ok(value) = value.parse::<T>() {
          return Ok(value);
        } else {
          return Err("Unknown Parsing Error".to_string());
        }
      }
    } else {
      return Err("Invalid Resource".to_string());
    }
    return Err("Invalid Resource".to_string());
  }

  pub fn set_item<T>(&mut self, item: T, count: u8) -> Result<(), String>
  where
    T: AsRef<str>,
  {
    let item = item.as_ref().to_string();
    if self
      .database
      .items
      .iter()
      .map(|i| i.id.clone())
      .collect::<String>()
      .contains(&item)
    {
      self.game_state.items.insert(item, count);
      Ok(())
    } else {
      Err("Invalid Item".to_string())
    }
  }

  pub fn set_time_of_day(&mut self, time_of_day: TimeOfDay) {
    self.game_state.time_of_day = time_of_day;
  }

  pub fn set_tile<T>(&mut self, tile: T) -> Result<(), String>
  where
    T: AsRef<str>,
  {
    let tile = tile.as_ref().to_string();

    if self.database.config.tiles.contains(&tile) {
      self.game_state.tile = tile;
      Ok(())
    } else {
      Err("Invalid Tile".to_string())
    }
  }

  pub fn get_reputation<T>(&self, reputation_id: T) -> Result<i32, String>
  where
    T: AsRef<str>,
  {
    let reputation = self.game_state.reputations.get(reputation_id.as_ref());
    match reputation {
      Some(reputation) => Ok(*reputation),
      None => {
        validate_reputation(&self.database.config.reputations, reputation_id)?;
        Ok(0)
      }
    }
  }

  pub fn decision_taken<T>(&self, decision_id: T) -> bool
  where
    T: AsRef<str>,
  {
    self
      .game_state
      .decisions
      .contains(&decision_id.as_ref().to_string())
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

  fn add_reputation<T>(&mut self, reputation_id: T, amount: i32) -> Result<(), String>
  where
    T: AsRef<str>,
  {
    validate_reputation(&self.database.config.reputations, &reputation_id)?;
    let reputation_id = reputation_id.as_ref();
    let reputation = self.game_state.reputations.get(reputation_id);
    match reputation {
      Some(reputation) => self
        .game_state
        .reputations
        .insert(reputation_id.to_string(), reputation + amount),
      None => self
        .game_state
        .reputations
        .insert(reputation_id.to_string(), amount),
    };
    Ok(())
  }

  fn set_decision<T>(&mut self, decision_id: T)
  where
    T: AsRef<str>,
  {
    self
      .game_state
      .decisions
      .push(decision_id.as_ref().to_string());
  }

  fn random_result(&mut self) -> Option<EventResult> {

    self.state.current_result = None;

    let choice = current_choice(self)?.clone();
    let frequencies = choice.results.iter().map(|r| r.chance).collect();
    let frequencies = frequency_sum(frequencies);
    let max = frequencies.last()?;
    let num = random_with_max(self, *max)?;

    let index = index_for_freq(num, frequencies);
    let result = choice.results.get(index)?;
    for modifier in &result.modifiers {
      if let Ok(()) = self.apply_modifier(modifier) {}
    }

    self.state.current_result = Some(index);
    Some(result.clone())
  }

  fn apply_modifier(&mut self, modifier: &cuentitos_common::Modifier) -> Result<(), String> {
    match modifier {
      Modifier::Reputation { id, amount } => match amount.parse::<i32>() {
        Ok(amount) => self.add_reputation(id, amount),
        Err(error) => Err(format!("Couldn't parse amount: {}", error)),
      },
      Modifier::Decision(id) => Ok(self.set_decision(id)),
      _ => Ok(()),
    }
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
    let mut result = true;

    for requirement in &event.requirements {
      result = result && self.requirement_met(requirement)
    }

    result
  }

  fn requirement_met(&self, requirement: &EventRequirement) -> bool {
    match requirement {
      EventRequirement::Resource {
        resource,
        condition,
        amount,
      } => {
        let current_value = self
          .game_state
          .resources
          .get(&resource.id)
          .unwrap_or(&"0".to_string())
          .clone();
        match resource.kind {
          ResourceKind::Integer => {
            let cv = current_value.parse::<i32>().unwrap_or(0);
            let a = amount.parse::<i32>().unwrap_or(0);
            match condition {
              Condition::Equals => return cv == a,
              Condition::HigherThan => return cv > a,
              Condition::LessThan => return cv < a,
              _ => return true,
            }
          }
          ResourceKind::Float => {
            let cv = current_value.parse::<f32>().unwrap_or(0.0);
            let a = amount.parse::<f32>().unwrap_or(0.0);
            match condition {
              Condition::Equals => return cv == a,
              Condition::HigherThan => return cv > a,
              Condition::LessThan => return cv < a,
              _ => return true,
            }
          }
          ResourceKind::Bool => {
            let cv = current_value.parse::<bool>().unwrap_or(false);
            let a = amount.parse::<bool>().unwrap_or(false);

            if condition == &Condition::Equals {
              return cv == a;
            }
            return true;
          }
        }
      }
      EventRequirement::Item {
        id,
        condition,
        amount,
      } => {
        let cv = *self.game_state.items.get(id).unwrap_or(&0);
        let a = amount.parse::<u8>().unwrap_or(0);
        match condition {
          Condition::Equals => return cv == a,
          Condition::HigherThan => return cv > a,
          Condition::LessThan => return cv < a,
          _ => return true,
        }
      }
      EventRequirement::Reputation {
        id,
        condition,
        amount,
      } => {
        let cv = *self.game_state.reputations.get(id).unwrap_or(&0);
        let a = amount.parse::<i32>().unwrap_or(0);
        match condition {
          Condition::Equals => return cv == a,
          Condition::HigherThan => return cv > a,
          Condition::LessThan => return cv < a,
          _ => return true,
        }
      }
      EventRequirement::TimeOfDay { id, condition } => {
        let cv = self.game_state.time_of_day.clone();
        match condition {
          Condition::Equals => return cv == *id,
          Condition::MutEx => return cv != *id,
          _ => return true,
        }
      }
      EventRequirement::Decision { id, condition } => match condition {
        Condition::Depends => return self.game_state.decisions.contains(id),
        Condition::MutEx => return !self.game_state.decisions.contains(id),
        _ => return true,
      },
      EventRequirement::Event { id, condition } => match condition {
        Condition::Depends => return self.state.previous_events.contains(id),
        Condition::MutEx => return !self.state.previous_events.contains(id),
        _ => return true,
      },
      EventRequirement::Tile { id, condition } => match condition {
        Condition::Equals => return self.game_state.tile == *id,
        Condition::MutEx => return self.game_state.tile != *id,
        _ => return true,
      },
      EventRequirement::Empty => true,
    }
  }

  fn event_frequencies(&self) -> Vec<u32> {
    let mut result = vec![];

    for idx in self.available_events() {
      let mut freq: u32 = 50;
      for requirement in &self.database.events[self.id_to_index[&idx]].requirements {
        if self.requirement_met(&requirement) {
          freq += self.database.config.runtime.met_requirement_frequency_boost;
        }
      }
      result.push(freq);
    }

    result
  }
}

fn validate_reputation<T>(reputations: &Vec<ReputationId>, reputation_id: T) -> Result<(), String>
where
  T: AsRef<str>,
{
  if reputations.iter().any(|rep| rep == reputation_id.as_ref()) {
    Ok(())
  } else {
    Err("Invalid Reputation".to_string())
  }
}

fn random_with_max(runtime: &mut Runtime, max: u32) -> Option<u32> {
  if runtime.rng == None {
    runtime.rng = Some(Pcg32::from_entropy())
  }

  let mut rng = runtime.rng.as_ref()?.clone();
  let num = rng.gen_range(0..max);

  runtime.rng = Some(rng);
  Some(num)
}

fn frequency_sum(frequencies: Vec<u32>) -> Vec<u32> {
  let mut result = vec![];

  for freq in frequencies {
    let prev = result.last().unwrap_or(&0);
    result.push(prev + freq)
  }

  result
}

fn index_for_freq(num: u32, frequencies: Vec<u32>) -> usize {
  let mut index = 0;
  for freq in frequencies {
    if num <= freq {
      return index;
    } else {
      index += 1;
    }
  }
  return index;
}

fn current_event(runtime: &Runtime) -> Option<&Event> {
  let current_event = runtime.state.current_event?;
  runtime.database.events.get(current_event)
}

fn current_choice(runtime: &Runtime) -> Option<&EventChoice> {
  let event = current_event(runtime)?;
  let choice_id = runtime.state.current_choice?;
  event.choices.get(choice_id)
}

fn current_result(runtime: &Runtime) -> Option<&EventResult> {
  let choice = current_choice(runtime)?;
  let result_id = runtime.state.current_result?;
  choice.results.get(result_id)
}

#[cfg(test)]
mod test {
  use std::default;

use crate::runtime::test::test_utils::serialize;
  use crate::runtime::Runtime;
  use crate::runtime_datatypes;
  use crate::GameState;
  use cuentitos_common::test_utils::load_mp_fixture;
  use cuentitos_common::Database;
  use cuentitos_common::Event;
  use cuentitos_common::*;

  #[test]
  fn new_runtime_accepts_database() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database)
  }

  #[test]
  fn runtime_can_be_saved_and_loaded() {
    // TODO(fran): fix this
    // let mut path = std::env::temp_dir().to_path_buf();
    // path.push("state_save.mp");

    // let db = load_mp_fixture("database").unwrap();
    // let database = Database::from_u8(&db).unwrap();
    // let runtime = Runtime::new(database.clone());

    // runtime.save(&path).unwrap();

    // let runtime2 = Runtime::load(path).unwrap();

    // assert_eq!(runtime, runtime2);
  }

  #[test]
  fn random_event_stores_it_in_current_and_previous_events() {
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
    runtime.set_seed(1);

    runtime.next_event().unwrap();

    assert_eq!(runtime.state.current_event, Some(0));

    assert!(runtime
      .state
      .previous_event_cooldown
      .contains_key("event-1"));
    runtime.next_event().unwrap();
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
    runtime.set_seed(1);
    runtime.next_event().unwrap();
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
    runtime.set_seed(1);

    let game_state = GameState::default();
    runtime.game_state = game_state;

    assert_eq!(runtime.available_events(), ["event-1", "event-2"]);
    assert_eq!(runtime.event_frequencies(), [50, 50]);

    runtime.next_event().unwrap();
    assert_eq!(runtime.available_events(), ["event-2"]);
    assert_eq!(runtime.event_frequencies(), [50]);

    runtime.next_event().unwrap();
    assert!(runtime.available_events().is_empty());
    assert!(runtime.event_frequencies().is_empty());
    assert_eq!(runtime.next_event(), None);

    // Make sure that after a while it shows up again
    for _ in 0..7 {
      runtime.next_event();
    }

    let event = runtime.next_event().unwrap();
    assert_eq!(
      event,
      crate::Event::from_cuentitos(&runtime.database.events[0])
    );
    assert!(runtime.available_events().is_empty());
    assert!(runtime.event_frequencies().is_empty());
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

    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "12".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-integer-higher-than"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .resources
      .insert(resource, "10".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-integer-equals"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);
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
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "12.5".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-float-higher-than"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .resources
      .insert(resource, "10.5".to_string());
    assert_eq!(runtime.available_events(), ["event-resource-float-equals"]);

    assert_eq!(runtime.event_frequencies(), [100]);
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
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .resources
      .insert(resource.clone(), "false".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-resource-bool-equals-false"]
    );
    assert_eq!(runtime.event_frequencies(), [100]);
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

    runtime.game_state.items.insert(item.clone(), 1);
    assert_eq!(
      runtime.available_events(),
      ["event-item-less-than", "event-item-missing-less"]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime.game_state.items.insert(item.clone(), 15);
    assert_eq!(
      runtime.available_events(),
      ["event-item-higher-than", "event-item-missing-less"]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime.game_state.items.insert(item.clone(), 10);
    assert_eq!(
      runtime.available_events(),
      ["event-item-equals", "event-item-missing-less"]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);
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

    runtime.game_state.reputations.insert(reputation.clone(), 1);
    assert_eq!(
      runtime.available_events(),
      [
        "event-reputation-less-than",
        "event-reputation-missing-less"
      ]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime
      .game_state
      .reputations
      .insert(reputation.clone(), 15);
    assert_eq!(
      runtime.available_events(),
      [
        "event-reputation-higher-than",
        "event-reputation-missing-less"
      ]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime
      .game_state
      .reputations
      .insert(reputation.clone(), 10);
    assert_eq!(
      runtime.available_events(),
      ["event-reputation-equals", "event-reputation-missing-less"]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);
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
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime.game_state.time_of_day = TimeOfDay::Evening;
    assert_eq!(runtime.available_events(), ["event-tod-mutex-night"]);
    assert_eq!(runtime.event_frequencies(), [100]);

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
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime.game_state.decisions = vec!["decision".to_string()];
    assert_eq!(
      runtime.available_events(),
      ["event-decision", "event-decision-2"]
    );
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime.game_state.decisions = vec!["decision".to_string(), "decision-2".to_string()];
    assert_eq!(runtime.available_events(), ["event-decision"]);
    assert_eq!(runtime.event_frequencies(), [100]);
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
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime.state.previous_events = vec!["event-1".to_string()];
    assert_eq!(runtime.available_events(), ["event-depends", "event-mutex"]);
    assert_eq!(runtime.event_frequencies(), [100, 100]);

    runtime.state.previous_events = vec!["event-1".to_string(), "event-2".to_string()];
    assert_eq!(runtime.available_events(), ["event-depends"]);
    assert_eq!(runtime.event_frequencies(), [100]);
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
    assert_eq!(runtime.available_events(), ["event-tile-not-forest"]);
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime.game_state.tile = "forest".to_string();
    assert_eq!(runtime.available_events(), ["event-tile-forest"]);
    assert_eq!(runtime.event_frequencies(), [100]);
  }

  #[test]
  fn set_choice_with_valid_choice_works() {
    let db = Database {
      events: vec![Event {
        id: "event-1".to_string(),
        choices: vec![cuentitos_common::EventChoice {
         results: vec![cuentitos_common::EventResult{
          chance:1,
          ..Default::default()
         }],
         ..Default::default()
          }],
        ..Default::default()
      }],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    assert_eq!(runtime.state.current_event, None);

    runtime.next_event().unwrap();

    assert_eq!(runtime.state.current_event, Some(0));
    assert_eq!(runtime.state.current_choice, None);

    runtime.set_choice(0).unwrap();

    assert_eq!(runtime.state.current_choice, Some(0));
  }

  #[test]
  fn set_choice_when_requirements_not_met_choice_does_not_work() {
    let db = Database {
      events: vec![Event {
        id: "event-1".to_string(),
        choices: vec![cuentitos_common::EventChoice {
          requirements: vec![cuentitos_common::EventRequirement::Item {
            id: "my-item".to_string(),
            condition: cuentitos_common::Condition::Equals,
            amount: 1.to_string(),
          }],
          ..Default::default()
        }],
        ..Default::default()
      }],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);
    match runtime.set_choice(0) {
      Ok(_) => assert!(false),
      Err(string) => {
        assert_eq!(string, "No event has been drawn");
        assert_eq!(runtime.state.current_choice, None)
      }
    }

    runtime.next_event().unwrap();

    match runtime.set_choice(10) {
      Ok(_) => assert!(false),
      Err(string) => {
        assert_eq!(string, "Invalid choice");
        assert_eq!(runtime.state.current_choice, None)
      }
    }

    match runtime.set_choice(0) {
      Ok(_) => assert!(false),
      Err(string) => {
        assert_eq!(string, "Requirements for choice not met");
        assert_eq!(runtime.state.current_choice, None)
      }
    }
  }

  #[test]
  fn set_item_works() {
    let db = Database {
      items: vec![Item {
        id: "item".to_string(),
        ..Default::default()
      }],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    assert_eq!(runtime.game_state.items.get("item"), None);

    runtime.set_item("item", 3).unwrap();

    assert_eq!(*runtime.game_state.items.get("item").unwrap(), 3 as u8);

    assert_eq!(
      runtime.set_item("missing_item", 3),
      Err("Invalid Item".to_string())
    );
  }

  #[test]
  fn set_tile_works() {
    let db = Database {
      config: Config {
        tiles: vec!["forest".to_string()],
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);

    assert_eq!(runtime.game_state.tile, "");

    runtime.set_tile("forest").unwrap();

    assert_eq!(runtime.game_state.tile, "forest");

    assert_eq!(
      runtime.set_tile("missing_tile"),
      Err("Invalid Tile".to_string())
    );
  }

  #[test]
  fn set_game_state_resource() {
    let mut db = Database::default();
    db.config
      .resources
      .insert("resource-int".to_string(), ResourceKind::Integer);
    db.config
      .resources
      .insert("resource-float".to_string(), ResourceKind::Float);
    db.config
      .resources
      .insert("resource-bool".to_string(), ResourceKind::Bool);

    let mut runtime = Runtime::new(db);

    assert_eq!(
      runtime.set_resource("invalid".to_string(), 1),
      Err("Invalid Resource".to_string())
    );

    assert_eq!(
      runtime.set_resource("resource-int".to_string(), 10.5),
      Err("Invalid Resource Type".to_string())
    );
    assert_eq!(
      runtime.set_resource("resource-float".to_string(), true),
      Err("Invalid Resource Type".to_string())
    );
    assert_eq!(
      runtime.set_resource("resource-bool".to_string(), 10),
      Err("Invalid Resource Type".to_string())
    );

    runtime.set_resource("resource-int", 10).unwrap();
    runtime.set_resource("resource-float", 10.5 as f32).unwrap();
    runtime.set_resource("resource-bool", true).unwrap();

    assert_eq!(
      runtime.get_resource::<&str, i32>("resource-int").unwrap(),
      10
    );
    assert_eq!(
      runtime.get_resource::<&str, f32>("resource-float").unwrap(),
      10.5
    );
    assert_eq!(
      runtime.get_resource::<&str, bool>("resource-bool").unwrap(),
      true
    );
  }

  #[test]
  fn set_time_of_day() {
    let db = Database::default();
    let mut runtime = Runtime::new(db);

    runtime.set_time_of_day(TimeOfDay::Night);
    assert_eq!(runtime.game_state.time_of_day, TimeOfDay::Night);

    runtime.set_time_of_day(TimeOfDay::Noon);
    assert_eq!(runtime.game_state.time_of_day, TimeOfDay::Noon);

    runtime.set_time_of_day(TimeOfDay::Morning);
    assert_eq!(runtime.game_state.time_of_day, TimeOfDay::Morning);

    runtime.set_time_of_day(TimeOfDay::Evening);
    assert_eq!(runtime.game_state.time_of_day, TimeOfDay::Evening);
  }

  #[test]
  fn add_reputation() {
    let mut db = Database::default();
    db.config.reputations.push("reputation-1".to_string());

    let mut runtime = Runtime::new(db);

    match runtime.get_reputation(&"no-reputation".to_string()) {
      Ok(_) => assert!(false),
      Err(string) => {
        assert_eq!(string, "Invalid Reputation");
      }
    }

    match runtime.add_reputation(&"no-reputation".to_string(), 1) {
      Ok(_) => assert!(false),
      Err(string) => {
        assert_eq!(string, "Invalid Reputation");
      }
    }

    assert_eq!(
      runtime.get_reputation(&"reputation-1".to_string()).unwrap(),
      0
    );

    assert_eq!(
      runtime.add_reputation(&"reputation-1".to_string(), 1),
      Ok(())
    );
    assert_eq!(
      runtime.get_reputation(&"reputation-1".to_string()).unwrap(),
      1
    );

    assert_eq!(
      runtime.add_reputation(&"reputation-1".to_string(), -1),
      Ok(())
    );
    assert_eq!(
      runtime.get_reputation(&"reputation-1".to_string()).unwrap(),
      0
    );

    assert_eq!(
      runtime.add_reputation(&"reputation-1".to_string(), -1),
      Ok(())
    );
    assert_eq!(
      runtime.get_reputation(&"reputation-1".to_string()).unwrap(),
      -1
    );
  }

  #[test]
  fn current_modifiers_returns_modifiers_correctly() {
    let mut db = Database {
      events: vec![Event {
        id: "event-1".to_string(),
        choices: vec![cuentitos_common::EventChoice {
          results: vec![cuentitos_common::EventResult {
            chance: 100,
            modifiers: vec![
              Modifier::Resource {
                id: "resource-1".to_string(),
                amount: "1".to_string(),
              },
              Modifier::Resource {
                id: "resource-2".to_string(),
                amount: "-1".to_string(),
              },
              Modifier::Resource {
                id: "resource-3".to_string(),
                amount: "1.5".to_string(),
              },
              Modifier::Resource {
                id: "resource-4".to_string(),
                amount: "-1.5".to_string(),
              },
              Modifier::Resource {
                id: "resource-5".to_string(),
                amount: "true".to_string(),
              },
              Modifier::Resource {
                id: "resource-6".to_string(),
                amount: "false".to_string(),
              },
              Modifier::Item {
                id: "item-1".to_string(),
                amount: "1".to_string(),
              },
              Modifier::Item {
                id: "item-2".to_string(),
                amount: "-1".to_string(),
              },
              Modifier::Reputation {
                id: "reputation-1".to_string(),
                amount: "1".to_string(),
              },
              Modifier::Reputation {
                id: "reputation-2".to_string(),
                amount: "-1".to_string(),
              },
              Modifier::Decision("decision-1".to_string()),
              Modifier::Achievement("achievement-1".to_string()),
            ],
            ..Default::default()
          }],
          ..Default::default()
        }],
        ..Default::default()
      }],
      ..Default::default()
    };

    db.config
      .resources
      .insert("resource-1".to_string(), ResourceKind::Integer);
    db.config
      .resources
      .insert("resource-2".to_string(), ResourceKind::Integer);
    db.config
      .resources
      .insert("resource-3".to_string(), ResourceKind::Float);
    db.config
      .resources
      .insert("resource-4".to_string(), ResourceKind::Float);
    db.config
      .resources
      .insert("resource-5".to_string(), ResourceKind::Bool);
    db.config
      .resources
      .insert("resource-6".to_string(), ResourceKind::Bool);
    db.config.reputations.push("reputation-1".to_string());
    db.config.reputations.push("reputation-2".to_string());

    db.items.push(Item {
      id: "item-1".to_string(),
      ..Default::default()
    });
    db.items.push(Item {
      id: "item-2".to_string(),
      ..Default::default()
    });

    let mut runtime = Runtime::new(db);

    assert_eq!(
      runtime.next_event().unwrap(),
      crate::Event::from_cuentitos(&runtime.database.events[0])
    );

    runtime.set_choice(0).unwrap();
    let modifiers = runtime.current_modifiers().unwrap();

    assert_eq!(
      modifiers[0],
      crate::Modifier {
        kind: "resource".to_string(),
        id: "resource-1".to_string(),
        amount: "1".to_string(),
      }
    );

    assert_eq!(
      modifiers[1],
      crate::Modifier {
        kind: "resource".to_string(),
        id: "resource-2".to_string(),
        amount: "-1".to_string(),
      }
    );

    assert_eq!(
      modifiers[2],
      crate::Modifier {
        kind: "resource".to_string(),
        id: "resource-3".to_string(),
        amount: "1.5".to_string(),
      }
    );

    assert_eq!(
      modifiers[3],
      crate::Modifier {
        kind: "resource".to_string(),
        id: "resource-4".to_string(),
        amount: "-1.5".to_string(),
      }
    );

    assert_eq!(
      modifiers[4],
      crate::Modifier {
        kind: "resource".to_string(),
        id: "resource-5".to_string(),
        amount: "true".to_string(),
      }
    );

    assert_eq!(
      modifiers[5],
      crate::Modifier {
        kind: "resource".to_string(),
        id: "resource-6".to_string(),
        amount: "false".to_string(),
      }
    );

    assert_eq!(
      modifiers[6],
      crate::Modifier {
        kind: "item".to_string(),
        id: "item-1".to_string(),
        amount: "1".to_string(),
      }
    );

    assert_eq!(
      modifiers[7],
      crate::Modifier {
        kind: "item".to_string(),
        id: "item-2".to_string(),
        amount: "-1".to_string(),
      }
    );

    assert_eq!(
      modifiers[8],
      crate::Modifier {
        kind: "reputation".to_string(),
        id: "reputation-1".to_string(),
        amount: "1".to_string(),
      }
    );

    assert_eq!(
      modifiers[9],
      crate::Modifier {
        kind: "reputation".to_string(),
        id: "reputation-2".to_string(),
        amount: "-1".to_string(),
      }
    );

    assert_eq!(
      modifiers[10],
      crate::Modifier {
        kind: "decision".to_string(),
        id: "decision-1".to_string(),
        amount: "".to_string(),
      }
    );

    assert_eq!(
      modifiers[11],
      crate::Modifier {
        kind: "achievement".to_string(),
        id: "achievement-1".to_string(),
        amount: "".to_string(),
      }
    );
  }

  #[test]
  fn when_choice_has_modifications_on_reputation_and_decisions_they_are_computed() {
    let mut db = Database {
      events: vec![Event {
        id: "event-1".to_string(),
        choices: vec![cuentitos_common::EventChoice {
          results: vec![cuentitos_common::EventResult {
            chance: 100,
            modifiers: vec![
              Modifier::Reputation {
                id: "reputation-1".to_string(),
                amount: "1".to_string(),
              },
              Modifier::Reputation {
                id: "reputation-2".to_string(),
                amount: "-1".to_string(),
              },
              Modifier::Decision("decision-1".to_string()),
            ],
            ..Default::default()
          }],
          ..Default::default()
        }],
        ..Default::default()
      }],
      ..Default::default()
    };

    db.config.reputations.push("reputation-1".to_string());
    db.config.reputations.push("reputation-2".to_string());

    let mut runtime = Runtime::new(db);

    assert_eq!(runtime.get_reputation("reputation-1"), Ok(0));
    assert_eq!(runtime.get_reputation("reputation-2"), Ok(0));
    assert_eq!(runtime.decision_taken("decision-1"), false);

    assert_eq!(
      runtime.next_event().unwrap(),
      crate::Event::from_cuentitos(&runtime.database.events[0])
    );

    runtime.set_choice(0).unwrap();
    assert_eq!(runtime.get_reputation("reputation-1"), Ok(1));
    assert_eq!(runtime.get_reputation("reputation-2"), Ok(-1));
    assert_eq!(runtime.decision_taken("decision-1"), true);
  }
}
