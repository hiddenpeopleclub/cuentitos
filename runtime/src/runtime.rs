use cuentitos_common::EventChoice;
use cuentitos_common::EventResult;
use cuentitos_common::Modifier;
use cuentitos_common::ReputationId;
use cuentitos_common::TimeOfDay;

use cuentitos_common::Condition;
use cuentitos_common::Event;
use cuentitos_common::EventRequirement;
use cuentitos_common::VariableKind;
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

    runtime.state.current_locale = runtime.database.i18n.default_locale.clone();

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
            return self.load_event(event_id);
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

  pub fn load_event<T>(&mut self, event_id: T) -> Option<crate::Event>
  where
    T: AsRef<str>,
  {
    let event_id = event_id.as_ref().to_string();

    let index = *self.id_to_index.get(&event_id)?;
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

    Some(crate::Event::from_cuentitos(
      event,
      &self.database.i18n,
      &self.state.current_locale,
    ))
  }

  pub fn set_seed(&mut self, seed: u64) {
    self.seed = seed;
    self.rng = Some(Pcg32::seed_from_u64(seed));
  }

  pub fn set_locale<T>(&mut self, locale: T) -> Result<(), String>
  where
    T: AsRef<str>,
  {
    let locale = locale.as_ref().to_string();
    if self.database.i18n.has_locale(&locale) {
      self.state.current_locale = locale;
      return Ok(());
    } else {
      return Err("Missing Locale".to_string());
    }
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
          if let Some(result) = self.random_result() {
            if let Some(modifiers) = self.current_modifiers() {
              let event_result = crate::EventResult::from_cuentitos(
                &result,
                &self.database.i18n,
                &self.state.current_locale,
                modifiers,
              );
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
        Modifier::Variable { id, value } => v.push(crate::Modifier {
          kind: "variable".to_string(),
          id,
          value,
        }),
        Modifier::Item { id, value } => v.push(crate::Modifier {
          kind: "item".to_string(),
          id,
          value,
        }),
        Modifier::Reputation { id, value } => v.push(crate::Modifier {
          kind: "reputation".to_string(),
          id,
          value,
        }),
        Modifier::Decision(id) => v.push(crate::Modifier {
          kind: "decision".to_string(),
          id,
          value: String::default(),
        }),
        Modifier::Achievement(id) => v.push(crate::Modifier {
          kind: "achievement".to_string(),
          id,
          value: String::default(),
        }),
        _ => {}
      }
    }

    Some(v)
  }

  pub fn set_variable<R, T>(&mut self, variable: R, value: T) -> Result<(), String>
  where
    T: Display,
    R: AsRef<str>,
  {
    let variable = variable.as_ref().to_string();
    if self.database.config.variables.contains_key(&variable) {
      let t = std::any::type_name::<T>();
      if (t == "i32" && self.database.config.variables[&variable] == VariableKind::Integer)
        || (t == "f32" && self.database.config.variables[&variable] == VariableKind::Float)
        || (t == "bool" && self.database.config.variables[&variable] == VariableKind::Bool)
      {
        self
          .game_state
          .variables
          .insert(variable, value.to_string());
      } else {
        return Err("Invalid Variable Type".to_string());
      }
    } else {
      return Err("Invalid Variable".to_string());
    }
    Ok(())
  }

  pub fn get_variable_kind<R>(&self, variable: R) -> Option<VariableKind>
  where
    R: AsRef<str>,
  {
    let variable = variable.as_ref();

    if self.database.config.variables.contains_key(variable) {
      Some(self.database.config.variables[variable].clone())
    } else {
      None
    }
  }

  pub fn get_variable<R, T>(&self, variable: R) -> Result<T, String>
  where
    T: Display + std::str::FromStr + Default,
    R: AsRef<str>,
  {
    let variable = variable.as_ref().to_string();
    if self.database.config.variables.contains_key(&variable) {
      let t = std::any::type_name::<T>();

      if (t == "i32" && self.database.config.variables[&variable] == VariableKind::Integer)
        || (t == "f32" && self.database.config.variables[&variable] == VariableKind::Float)
        || (t == "bool" && self.database.config.variables[&variable] == VariableKind::Bool)
      {
        let value = match self.game_state.variables.get(&variable) {
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
      return Err("Invalid Variable".to_string());
    }
    return Err("Invalid Variable".to_string());
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

  fn add_reputation<T>(&mut self, reputation_id: T, value: i32) -> Result<(), String>
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
        .insert(reputation_id.to_string(), reputation + value),
      None => self
        .game_state
        .reputations
        .insert(reputation_id.to_string(), value),
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
      Modifier::Reputation { id, value } => match value.parse::<i32>() {
        Ok(value) => self.add_reputation(id, value),
        Err(error) => Err(format!("Couldn't parse value: {}", error)),
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

        self.variable_requirements_met(event)
      {
        result.push(event.id.clone());
      }
    }
    result
  }

  fn variable_requirements_met(&self, event: &Event) -> bool {
    let mut result = true;

    for requirement in &event.requirements {
      result = result && self.requirement_met(requirement)
    }

    result
  }

  fn requirement_met(&self, requirement: &EventRequirement) -> bool {
    match requirement {
      EventRequirement::Variable {
        variable,
        condition,
        value,
      } => {
        let current_value = self
          .game_state
          .variables
          .get(&variable.id)
          .unwrap_or(&"0".to_string())
          .clone();
        match &variable.kind {
          VariableKind::Integer => {
            let cv = current_value.parse::<i32>().unwrap_or(0);
            let a = value.parse::<i32>().unwrap_or(0);
            match condition {
              Condition::Equals => return cv == a,
              Condition::HigherThan => return cv > a,
              Condition::LessThan => return cv < a,
              _ => return true,
            }
          }
          VariableKind::Float => {
            let cv = current_value.parse::<f32>().unwrap_or(0.0);
            let a = value.parse::<f32>().unwrap_or(0.0);
            match condition {
              Condition::Equals => return cv == a,
              Condition::HigherThan => return cv > a,
              Condition::LessThan => return cv < a,
              _ => return true,
            }
          }
          VariableKind::Bool => {
            let cv = current_value.parse::<bool>().unwrap_or(false);
            let a = value.parse::<bool>().unwrap_or(false);

            if condition == &Condition::Equals {
              return cv == a;
            }
            return true;
          }
          VariableKind::Enum { .. } => {
            if condition == &Condition::Equals {
              return current_value == *value
            } else {
              return current_value != *value
            }
          }
        }
      }
      EventRequirement::Item {
        id,
        condition,
        value,
      } => {
        let cv = *self.game_state.items.get(id).unwrap_or(&0);
        let a = value.parse::<u8>().unwrap_or(0);
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
        value,
      } => {
        let cv = *self.game_state.reputations.get(id).unwrap_or(&0);
        let a = value.parse::<i32>().unwrap_or(0);
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
  use crate::runtime::Runtime;
  use crate::GameState;
  use cuentitos_common::Database;
  use cuentitos_common::Event;
  use cuentitos_common::*;
  use std::collections::HashMap;

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

    let event = runtime.next_event();

    assert_ne!(event, None);

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
      crate::Event::from_cuentitos(
        &runtime.database.events[0],
        &runtime.database.i18n,
        &runtime.state.current_locale
      )
    );
    assert!(runtime.available_events().is_empty());
    assert!(runtime.event_frequencies().is_empty());
  }

  #[test]
  fn requirements_on_integer_variable_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-variable-integer-higher-than".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Integer,
            },
            condition: Condition::HigherThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-variable-integer-less-than".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Integer,
            },
            condition: Condition::LessThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-variable-integer-equals".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Integer,
            },
            condition: Condition::Equals,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let variable = "variable-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "2".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-integer-less-than"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "12".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-integer-higher-than"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .variables
      .insert(variable, "10".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-integer-equals"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);
  }

  #[test]
  fn requirements_on_float_variable_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-variable-float-higher-than".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Float,
            },
            condition: Condition::HigherThan,
            value: "10.5".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-variable-float-less-than".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Float,
            },
            condition: Condition::LessThan,
            value: "10.5".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-variable-float-equals".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Float,
            },
            condition: Condition::Equals,
            value: "10.5".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let variable = "variable-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "2.5".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-float-less-than"]
    );
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "12.5".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-float-higher-than"]
    );

    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .variables
      .insert(variable, "10.5".to_string());
    assert_eq!(runtime.available_events(), ["event-variable-float-equals"]);

    assert_eq!(runtime.event_frequencies(), [100]);
  }

  #[test]
  fn requirements_on_bool_variable_are_honored() {
    let db = Database {
      events: vec![
        Event {
          id: "event-variable-bool-equals-true".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Bool,
            },
            condition: Condition::Equals,
            value: "true".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-variable-bool-equals-false".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Bool,
            },
            condition: Condition::Equals,
            value: "false".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let variable = "variable-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "true".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-bool-equals-true"]
    );
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "false".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-bool-equals-false"]
    );
    assert_eq!(runtime.event_frequencies(), [100]);
  }

  #[test]
  fn requirements_on_enum_variable_are_honored() {
    let values = vec![ "value".to_string(), "another-value".to_string() ];
    let db = Database {
      events: vec![
        Event {
          id: "event-variable-bool-equals-value".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Enum { values: values.clone() },
            },
            condition: Condition::Equals,
            value: "value".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-variable-bool-equals-another-value".to_string(),
          requirements: vec![EventRequirement::Variable {
            variable: Variable {
              id: "variable-1".to_string(),
              kind: VariableKind::Enum { values: values.clone() },
            },
            condition: Condition::Equals,
            value: "another-value".to_string(),
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };

    let variable = "variable-1".to_string();

    let mut runtime = Runtime::new(db);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "value".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-bool-equals-value"]
    );
    assert_eq!(runtime.event_frequencies(), [100]);

    runtime
      .game_state
      .variables
      .insert(variable.clone(), "another-value".to_string());
    assert_eq!(
      runtime.available_events(),
      ["event-variable-bool-equals-another-value"]
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
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-higher-than".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "item".to_string(),
            condition: Condition::HigherThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-less-than".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "item".to_string(),
            condition: Condition::LessThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-missing-higher".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "missing".to_string(),
            condition: Condition::HigherThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-item-missing-less".to_string(),
          requirements: vec![EventRequirement::Item {
            id: "missing".to_string(),
            condition: Condition::LessThan,
            value: "10".to_string(),
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
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-higher-than".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "reputation".to_string(),
            condition: Condition::HigherThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-less-than".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "reputation".to_string(),
            condition: Condition::LessThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-missing-higher".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "missing".to_string(),
            condition: Condition::HigherThan,
            value: "10".to_string(),
          }],
          ..Default::default()
        },
        Event {
          id: "event-reputation-missing-less".to_string(),
          requirements: vec![EventRequirement::Reputation {
            id: "missing".to_string(),
            condition: Condition::LessThan,
            value: "10".to_string(),
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
  fn set_choice_with_valid_choice_works() {
    let db = Database {
      events: vec![Event {
        id: "event-1".to_string(),
        choices: vec![cuentitos_common::EventChoice {
          results: vec![cuentitos_common::EventResult {
            chance: 1,
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
            value: 1.to_string(),
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
  fn set_game_state_variable() {
    let mut db = Database::default();
    db.config
      .variables
      .insert("variable-int".to_string(), VariableKind::Integer);
    db.config
      .variables
      .insert("variable-float".to_string(), VariableKind::Float);
    db.config
      .variables
      .insert("variable-bool".to_string(), VariableKind::Bool);

    let mut runtime = Runtime::new(db);

    assert_eq!(
      runtime.set_variable("invalid".to_string(), 1),
      Err("Invalid Variable".to_string())
    );

    assert_eq!(
      runtime.set_variable("variable-int".to_string(), 10.5),
      Err("Invalid Variable Type".to_string())
    );
    assert_eq!(
      runtime.set_variable("variable-float".to_string(), true),
      Err("Invalid Variable Type".to_string())
    );
    assert_eq!(
      runtime.set_variable("variable-bool".to_string(), 10),
      Err("Invalid Variable Type".to_string())
    );

    runtime.set_variable("variable-int", 10).unwrap();
    runtime.set_variable("variable-float", 10.5 as f32).unwrap();
    runtime.set_variable("variable-bool", true).unwrap();

    assert_eq!(
      runtime.get_variable::<&str, i32>("variable-int").unwrap(),
      10
    );
    assert_eq!(
      runtime.get_variable::<&str, f32>("variable-float").unwrap(),
      10.5
    );
    assert_eq!(
      runtime.get_variable::<&str, bool>("variable-bool").unwrap(),
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
              Modifier::Variable {
                id: "variable-1".to_string(),
                value: "1".to_string(),
              },
              Modifier::Variable {
                id: "variable-2".to_string(),
                value: "-1".to_string(),
              },
              Modifier::Variable {
                id: "variable-3".to_string(),
                value: "1.5".to_string(),
              },
              Modifier::Variable {
                id: "variable-4".to_string(),
                value: "-1.5".to_string(),
              },
              Modifier::Variable {
                id: "variable-5".to_string(),
                value: "true".to_string(),
              },
              Modifier::Variable {
                id: "variable-6".to_string(),
                value: "false".to_string(),
              },
              Modifier::Item {
                id: "item-1".to_string(),
                value: "1".to_string(),
              },
              Modifier::Item {
                id: "item-2".to_string(),
                value: "-1".to_string(),
              },
              Modifier::Reputation {
                id: "reputation-1".to_string(),
                value: "1".to_string(),
              },
              Modifier::Reputation {
                id: "reputation-2".to_string(),
                value: "-1".to_string(),
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
      .variables
      .insert("variable-1".to_string(), VariableKind::Integer);
    db.config
      .variables
      .insert("variable-2".to_string(), VariableKind::Integer);
    db.config
      .variables
      .insert("variable-3".to_string(), VariableKind::Float);
    db.config
      .variables
      .insert("variable-4".to_string(), VariableKind::Float);
    db.config
      .variables
      .insert("variable-5".to_string(), VariableKind::Bool);
    db.config
      .variables
      .insert("variable-6".to_string(), VariableKind::Bool);
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
      crate::Event::from_cuentitos(
        &runtime.database.events[0],
        &runtime.database.i18n,
        &runtime.state.current_locale
      )
    );

    runtime.set_choice(0).unwrap();
    let modifiers = runtime.current_modifiers().unwrap();

    assert_eq!(
      modifiers[0],
      crate::Modifier {
        kind: "variable".to_string(),
        id: "variable-1".to_string(),
        value: "1".to_string(),
      }
    );

    assert_eq!(
      modifiers[1],
      crate::Modifier {
        kind: "variable".to_string(),
        id: "variable-2".to_string(),
        value: "-1".to_string(),
      }
    );

    assert_eq!(
      modifiers[2],
      crate::Modifier {
        kind: "variable".to_string(),
        id: "variable-3".to_string(),
        value: "1.5".to_string(),
      }
    );

    assert_eq!(
      modifiers[3],
      crate::Modifier {
        kind: "variable".to_string(),
        id: "variable-4".to_string(),
        value: "-1.5".to_string(),
      }
    );

    assert_eq!(
      modifiers[4],
      crate::Modifier {
        kind: "variable".to_string(),
        id: "variable-5".to_string(),
        value: "true".to_string(),
      }
    );

    assert_eq!(
      modifiers[5],
      crate::Modifier {
        kind: "variable".to_string(),
        id: "variable-6".to_string(),
        value: "false".to_string(),
      }
    );

    assert_eq!(
      modifiers[6],
      crate::Modifier {
        kind: "item".to_string(),
        id: "item-1".to_string(),
        value: "1".to_string(),
      }
    );

    assert_eq!(
      modifiers[7],
      crate::Modifier {
        kind: "item".to_string(),
        id: "item-2".to_string(),
        value: "-1".to_string(),
      }
    );

    assert_eq!(
      modifiers[8],
      crate::Modifier {
        kind: "reputation".to_string(),
        id: "reputation-1".to_string(),
        value: "1".to_string(),
      }
    );

    assert_eq!(
      modifiers[9],
      crate::Modifier {
        kind: "reputation".to_string(),
        id: "reputation-2".to_string(),
        value: "-1".to_string(),
      }
    );

    assert_eq!(
      modifiers[10],
      crate::Modifier {
        kind: "decision".to_string(),
        id: "decision-1".to_string(),
        value: "".to_string(),
      }
    );

    assert_eq!(
      modifiers[11],
      crate::Modifier {
        kind: "achievement".to_string(),
        id: "achievement-1".to_string(),
        value: "".to_string(),
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
                value: "1".to_string(),
              },
              Modifier::Reputation {
                id: "reputation-2".to_string(),
                value: "-1".to_string(),
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
      crate::Event::from_cuentitos(
        &runtime.database.events[0],
        &runtime.database.i18n,
        &runtime.state.current_locale
      )
    );

    runtime.set_choice(0).unwrap();
    assert_eq!(runtime.get_reputation("reputation-1"), Ok(1));
    assert_eq!(runtime.get_reputation("reputation-2"), Ok(-1));
    assert_eq!(runtime.decision_taken("decision-1"), true);
  }

  #[test]
  fn set_locale_with_existing_locale_works() {
    let database = Database {
      i18n: I18n {
        locales: vec!["en".to_string(), "es".to_string()],
        default_locale: "en".to_string(),
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime::new(database.clone());

    assert_eq!(runtime.state.current_locale, "en".to_string());
    runtime.set_locale("es").unwrap();
    assert_eq!(runtime.state.current_locale, "es".to_string());
  }

  #[test]
  fn set_locale_with_wrong_locale_fails() {
    let database = Database {
      i18n: I18n {
        locales: vec!["en".to_string(), "es".to_string()],
        default_locale: "en".to_string(),
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime::new(database.clone());

    assert_eq!(
      runtime.set_locale("missing"),
      Err("Missing Locale".to_string())
    );
  }

  #[test]
  fn event_workflow_supports_i18n() {
    let mut event_strings: LanguageDb = HashMap::new();
    event_strings.insert("event-1-title".to_string(), "a title".to_string());
    event_strings.insert(
      "event-1-description".to_string(),
      "a description".to_string(),
    );
    event_strings.insert("event-1-choice-0".to_string(), "a choice".to_string());
    event_strings.insert(
      "event-1-choice-0-result-0-text".to_string(),
      "a result".to_string(),
    );

    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::new();
    strings.insert("en".to_string(), event_strings);

    let db = Database {
      events: vec![Event {
        id: "1".to_string(),
        title: "event-1-title".to_string(),
        description: "event-1-description".to_string(),
        choices: vec![EventChoice {
          text: "event-1-choice-0".to_string(),
          results: vec![EventResult {
            chance: 100,
            text: "event-1-choice-0-result-0-text".to_string(),
            ..Default::default()
          }],
          ..Default::default()
        }],
        ..Default::default()
      }],
      i18n: I18n {
        locales: vec!["en".to_string()],
        default_locale: "en".to_string(),
        strings,
      },
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);
    runtime.set_seed(1);

    let event = runtime.next_event().unwrap();

    assert_eq!(event.title, "a title");
    assert_eq!(event.description, "a description");
    assert_eq!(event.choices[0].text, "a choice");

    let result = runtime.set_choice(0).unwrap();
    assert_eq!(result.text, "a result");
  }

  #[test]
  fn load_event_works() {
    let db = Database {
      events: vec![Event {
        id: "event-1".to_string(),
        ..Default::default()
      }],
      ..Default::default()
    };

    let mut runtime = Runtime::new(db);
    runtime.load_event("event-1");

    assert_eq!(runtime.state.current_event.unwrap(), 0);
  }
}
