use std::fmt::Display;
use std::println;

use crate::GameState;
use cuentitos_common::BlockId;
use cuentitos_common::BlockSettings;
use cuentitos_common::Condition;
use cuentitos_common::Database;
use cuentitos_common::SectionKey;
use cuentitos_common::VariableKind;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Block {
  pub text: String,
  pub choices: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Runtime {
  pub database: Database,
  pub block_stack: Vec<BlockId>,
  pub choices: Vec<BlockId>,
  #[serde(skip)]
  pub game_state: GameState,
  #[serde(skip)]
  rng: Option<Pcg32>,
  seed: u64,
}

impl Runtime {
  pub fn new(database: Database) -> Runtime {
    let game_state: GameState = GameState::from_config(&database.config);
    Runtime {
      database,
      game_state,
      ..Default::default()
    }
  }

  pub fn set_seed(&mut self, seed: u64) {
    self.seed = seed;
    self.rng = Some(Pcg32::seed_from_u64(seed));
  }

  pub fn jump_to_section(&mut self, section: String, subsection: Option<String>) {
    let current_section = self.game_state.current_section.clone();
    if subsection.is_none() {
      if let Some(current_section) = current_section {
        let key = SectionKey {
          section: current_section,
          subsection: Some(section.clone()),
        };
        if let Some(block_id) = self.database.sections.get(&key) {
          self.block_stack.clear();
          self.push_stack(*block_id);
          return;
        }
      }
    }

    let key = SectionKey {
      section,
      subsection,
    };
    if let Some(block_id) = self.database.sections.get(&key) {
      self.block_stack.clear();
      self.game_state.current_section = Some(key.section);
      self.game_state.current_subsection = None;
      self.push_stack(*block_id);
    } else {
      println!("Can't find section: {:?}", key);
    }
  }

  pub fn next_block(&mut self) -> Option<Block> {
    if self.database.blocks.is_empty() {
      return None;
    }

    if !self.update_stack() {
      return None;
    }

    while !self.next_block_meets_requirements() {
      self.pop_stack_and_find_next();
    }
    self.get_next_block_output()
  }

  pub fn pick_choice(&mut self, choice: usize) -> Option<Block> {
    if self.database.blocks.is_empty() {
      return None;
    }

    let choices = &self.choices;

    if choices.is_empty() {
      println!("There are no choices");
      return None;
    }

    if choice >= choices.len() {
      println!("There's only {} options", choices.len());
      return None;
    }

    if choices[choice] >= self.database.blocks.len() {
      println!("Invalid option");
      return None;
    }

    self.push_stack(choices[choice]);
    self.next_block()
  }

  pub fn set_variable<R, T>(&mut self, variable: R, value: T) -> Result<(), String>
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
        || (t == "alloc::string::String"
          && self.database.config.variables[&variable] == VariableKind::String)
        || (t == "&str" && self.database.config.variables[&variable] == VariableKind::String)
        || self.is_valid_enum::<T>(&value.to_string())
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
    let value = match self.game_state.variables.get(&variable) {
      Some(value) => value.clone(),
      None => T::default().to_string(),
    };
    if self.database.config.variables.contains_key(&variable) {
      let t = std::any::type_name::<T>();
      if (t == "i32" && self.database.config.variables[&variable] == VariableKind::Integer)
        || (t == "f32" && self.database.config.variables[&variable] == VariableKind::Float)
        || (t == "bool" && self.database.config.variables[&variable] == VariableKind::Bool)
        || (t == "alloc::string::String"
          && self.database.config.variables[&variable] == VariableKind::String)
        || (t == "&str" && self.database.config.variables[&variable] == VariableKind::String)
        || self.is_valid_enum::<T>(&value)
      {
        if let Ok(value) = value.parse::<T>() {
          return Ok(value);
        } else {
          return Err("Unknown Parsing Error".to_string());
        }
      }
    } else {
      return Err("Invalid Variable".to_string());
    }

    Err("Invalid Variable".to_string())
  }

  fn next_block_meets_requirements(&mut self) -> bool {
    if let Some(id) = self.block_stack.last() {
      self.meets_requirements(*id)
    } else {
      false
    }
  }

  fn meets_requirements(&mut self, id: BlockId) -> bool {
    for requirement in &self.get_block(id).get_settings().requirements {
      if !self.meets_condition(&requirement.condition) {
        return false;
      }
    }
    self.roll_chances_for_block(id)
  }

  fn meets_condition(&self, condition: &Condition) -> bool {
    if let Some(kind) = self.get_variable_kind(condition.variable.clone()) {
      match kind {
        VariableKind::Integer => {
          if let Ok(current_value) = self.get_variable::<&str, i32>(&condition.variable) {
            if let Ok(condition_value) = condition.value.parse::<i32>() {
              match condition.operator {
                cuentitos_common::Operator::Equal => return current_value == condition_value,
                cuentitos_common::Operator::NotEqual => return current_value != condition_value,
                cuentitos_common::Operator::GreaterThan => return current_value > condition_value,
                cuentitos_common::Operator::LessThan => return current_value < condition_value,
                cuentitos_common::Operator::GreaterOrEqualThan => {
                  return current_value >= condition_value
                }
                cuentitos_common::Operator::LessOrEqualThan => {
                  return current_value <= condition_value
                }
              }
            }
          }
        }
        VariableKind::Float => {
          if let Ok(current_value) = self.get_variable::<&str, f32>(&condition.variable) {
            if let Ok(condition_value) = condition.value.parse::<f32>() {
              match condition.operator {
                cuentitos_common::Operator::Equal => return current_value == condition_value,
                cuentitos_common::Operator::NotEqual => return current_value != condition_value,
                cuentitos_common::Operator::GreaterThan => return current_value > condition_value,
                cuentitos_common::Operator::LessThan => return current_value < condition_value,
                cuentitos_common::Operator::GreaterOrEqualThan => {
                  return current_value >= condition_value
                }
                cuentitos_common::Operator::LessOrEqualThan => {
                  return current_value <= condition_value
                }
              }
            }
          }
        }
        VariableKind::Bool => {
          if let Ok(current_value) = self.get_variable::<&str, bool>(&condition.variable) {
            if let Ok(condition_value) = condition.value.parse::<bool>() {
              match condition.operator {
                cuentitos_common::Operator::Equal => return current_value == condition_value,
                cuentitos_common::Operator::NotEqual => return current_value != condition_value,
                _ => {}
              }
            }
          }
        }
        _ => {
          if let Ok(current_value) = self.get_variable::<&str, String>(&condition.variable) {
            if let Ok(condition_value) = condition.value.parse::<String>() {
              match condition.operator {
                cuentitos_common::Operator::Equal => return current_value == condition_value,
                cuentitos_common::Operator::NotEqual => return current_value != condition_value,
                _ => {}
              }
            }
          }
        }
      }
    }

    false
  }

  fn roll_chances_for_block(&mut self, id: BlockId) -> bool {
    match self.get_block(id).get_settings().chance {
      cuentitos_common::Chance::None => true,
      cuentitos_common::Chance::Frequency(_) => true,
      cuentitos_common::Chance::Probability(probability) => {
        if let Some(random_number) = self.random_float() {
          random_number < probability
        } else {
          false
        }
      }
    }
  }

  fn is_valid_enum<T>(&self, value: &String) -> bool
  where
    T: Display + std::str::FromStr + Default,
  {
    for kind in self.database.config.variables.values() {
      if let VariableKind::Enum(possible_values) = kind {
        let mut value_found = false;
        for possible_value in possible_values {
          if value == possible_value {
            value_found = true;
            break;
          }
        }

        if value_found {
          let mut all_values_parse = true;
          for possible_value in possible_values {
            if possible_value.parse::<T>().is_err() {
              all_values_parse = false;
              break;
            }
          }
          if all_values_parse {
            return true;
          }
        }
      }
    }

    false
  }

  fn random_float(&mut self) -> Option<f32> {
    if self.rng.is_none() {
      self.rng = Some(Pcg32::from_entropy())
    }

    let mut rng = self.rng.as_ref()?.clone();
    let num = rng.gen();

    self.rng = Some(rng);
    Some(num)
  }

  fn random_with_max(&mut self, max: u32) -> Option<u32> {
    if self.rng.is_none() {
      self.rng = Some(Pcg32::from_entropy())
    }

    let mut rng = self.rng.as_ref()?.clone();
    let num = rng.gen_range(0..max);

    self.rng = Some(rng);
    Some(num)
  }

  fn get_next_block_output(&mut self) -> Option<Block> {
    let id = self.block_stack.last().unwrap();
    let block = self.get_block(*id);
    if let cuentitos_common::Block::Text { id, settings: _ } = block {
      println!("USE I18n!!!");
      return Some(Block {
        text: id.clone(),
        choices: self.get_choices_strings(),
      });
    }

    None
  }

  fn get_frequency_with_modifier(&self, settings: &BlockSettings) -> u32 {
    match settings.chance {
      cuentitos_common::Chance::None => 0,
      cuentitos_common::Chance::Frequency(frequency) => {
        let mut final_frequency = frequency as i32;
        for freq_mod in &settings.frequency_modifiers {
          if self.meets_condition(&freq_mod.condition) {
            final_frequency += freq_mod.value;
          }
        }
        final_frequency as u32
      }
      cuentitos_common::Chance::Probability(_) => 0,
    }
  }
  fn apply_modifiers(&mut self) {
    let id = self.block_stack.last().unwrap();
    let block = self.get_block(*id);
    for modifier in block.get_settings().modifiers.clone() {
      match self.get_variable_kind(&modifier.variable) {
        Some(kind) => {
          let result = match kind {
            VariableKind::Integer => {
              let value = &modifier.added_value.parse::<i32>();
              match value {
                Ok(value) => {
                  if modifier.is_override {
                    self.set_variable(&modifier.variable, *value)
                  } else if let Ok(previous_value) =
                    self.get_variable::<&str, i32>(&modifier.variable)
                  {
                    self.set_variable(&modifier.variable, *value + previous_value)
                  } else {
                    self.set_variable(&modifier.variable, *value)
                  }
                }
                Err(e) => Err(e.to_string()),
              }
            }
            VariableKind::Float => {
              let value = &modifier.added_value.parse::<f32>();
              match value {
                Ok(value) => {
                  if modifier.is_override {
                    self.set_variable(&modifier.variable, *value)
                  } else if let Ok(previous_value) =
                    self.get_variable::<&str, f32>(&modifier.variable)
                  {
                    self.set_variable(&modifier.variable, *value + previous_value)
                  } else {
                    self.set_variable(&modifier.variable, *value)
                  }
                }
                Err(e) => Err(e.to_string()),
              }
            }
            VariableKind::Bool => {
              let value = &modifier.added_value.parse::<bool>();
              match value {
                Ok(value) => self.set_variable(&modifier.variable, *value),
                Err(e) => Err(e.to_string()),
              }
            }
            _ => self.set_variable(&modifier.variable, modifier.added_value),
          };

          if result.is_err() {
            println!("{}", result.unwrap_err());
          }
        }
        None => {
          println!(
            "Can't modify variable {:?} because it doesn't exists.",
            modifier.variable
          )
        }
      }
    }
  }

  fn push_stack(&mut self, id: BlockId) {
    self.block_stack.push(id);
    self.apply_modifiers();

    match self.get_block(id) {
      cuentitos_common::Block::Section { id, settings: _ } => {
        self.game_state.current_section = Some(id.clone());
        self.game_state.current_subsection = None;
      }
      cuentitos_common::Block::Subsection { id, settings: _ } => {
        self.game_state.current_subsection = Some(id.clone());
      }
      _ => {}
    }
  }

  fn update_stack(&mut self) -> bool {
    if self.block_stack.is_empty() {
      return self.push_first_element_in_stack();
    }

    let last_block_id = self.block_stack.last().unwrap();

    if last_block_id >= &self.database.blocks.len() {
      return false;
    }

    let settings = self.get_block(*last_block_id).get_settings().clone();

    if !settings.children.is_empty() {
      return self.push_first_child_in_stack(&settings);
    }

    if self.push_next_block() {
      true
    } else {
      self.pop_stack_and_find_next()
    }
  }

  fn push_first_child_in_stack(&mut self, settings: &BlockSettings) -> bool {
    match self.get_block(settings.children[0]) {
      cuentitos_common::Block::Text { id: _, settings: _ } => {
        self.push_stack(settings.children[0]);
        true
      }
      cuentitos_common::Block::Choice { id: _, settings: _ } => {
        println!("Make a choice");
        false
      }
      cuentitos_common::Block::Bucket { name: _, settings } => {
        if let Some(new_block) = self.get_random_block_from_bucket(&settings.clone()) {
          if let cuentitos_common::Block::Choice { id: _, settings: _ } = self.get_block(new_block)
          {
            println!("Make a choice");
            false
          } else {
            self.push_stack(new_block);
            true
          }
        } else {
          false
        }
      }
      _ => false,
    }
  }
  fn push_first_element_in_stack(&mut self) -> bool {
    self.push_stack(0);
    let last_block_id = *self.block_stack.last().unwrap();
    let last_block = self.get_block(last_block_id).clone();
    match last_block {
      cuentitos_common::Block::Section { id: _, settings: _ } => {
        return self.update_stack();
      }
      cuentitos_common::Block::Subsection { id: _, settings: _ } => {
        return self.update_stack();
      }
      _ => {}
    }
    true
  }

  fn push_next_block(&mut self) -> bool {
    let last_block_id = *self.block_stack.last().unwrap();

    let last_block = self.get_block(last_block_id).clone();
    let last_settings = last_block.get_settings();
    match &last_settings.next {
      cuentitos_common::NextBlock::None => false,
      cuentitos_common::NextBlock::BlockId(other_id) => {
        self.block_stack.pop();
        self.push_stack(*other_id);
        true
      }
      cuentitos_common::NextBlock::EndOfFile => {
        println!("Story finished\n");
        self.block_stack = vec![0];
        self.game_state = GameState::from_config(&self.database.config);
        true
      }
      cuentitos_common::NextBlock::Section(key) => {
        self.jump_to_section(key.section.clone(), key.subsection.clone());
        self.update_stack();
        true
      }
    }
  }
  fn pop_stack_and_find_next(&mut self) -> bool {
    let last_block_id = *self.block_stack.last().unwrap();
    self.block_stack.pop();
    if self.block_stack.is_empty() {
      self.push_stack(last_block_id + 1);
      let last_block_id = *self.block_stack.last().unwrap();
      let last_block = self.get_block(last_block_id).clone();
      match last_block {
        cuentitos_common::Block::Section { id: _, settings: _ } => {
          return self.update_stack();
        }
        cuentitos_common::Block::Subsection { id: _, settings: _ } => {
          return self.update_stack();
        }
        _ => {}
      }
      return true;
    }

    let parent = &self.database.blocks[*self.block_stack.last().unwrap()].clone();
    let parent_settings = parent.get_settings();
    let mut child_found = false;
    for child in &parent_settings.children {
      if child_found {
        match self.get_block(*child).clone() {
          cuentitos_common::Block::Text { id: _, settings: _ } => self.push_stack(*child),
          cuentitos_common::Block::Choice { id: _, settings: _ } => {
            continue;
          }
          cuentitos_common::Block::Bucket { name: _, settings } => {
            if let Some(new_block) = self.get_random_block_from_bucket(&settings.clone()) {
              self.push_stack(new_block);
              return true;
            }
          }
          _ => {
            continue;
          }
        }
        if let cuentitos_common::Block::Choice { id: _, settings: _ } = self.get_block(*child) {
          continue;
        }
        self.push_stack(*child);
        return true;
      }
      if *child == last_block_id {
        child_found = true;
      }
    }

    self.pop_stack_and_find_next()
  }

  fn get_random_block_from_bucket(&mut self, settings: &BlockSettings) -> Option<BlockId> {
    let mut total_frequency = 0;
    for child in &settings.children {
      if self.meets_requirements(*child) {
        let child_settings = self.get_block(*child).get_settings();
        let frequency = self.get_frequency_with_modifier(child_settings);
        total_frequency += frequency;
      }
    }

    //TODO remove unwrap
    let mut random_number = self.random_with_max(total_frequency).unwrap();

    for child in &settings.children {
      if self.meets_requirements(*child) {
        let child_settings = self.get_block(*child).get_settings();
        let frequency = self.get_frequency_with_modifier(child_settings);
        if random_number <= frequency {
          return Some(*child);
        }
        random_number -= frequency;
      }
    }
    None
  }

  fn get_block(&self, id: BlockId) -> &cuentitos_common::Block {
    &self.database.blocks[id]
  }

  fn get_choices_strings(&mut self) -> Vec<String> {
    self.update_choices();
    let mut choices_strings = Vec::default();
    println!("USE I18n!!!");
    for choice in &self.choices {
      if let cuentitos_common::Block::Choice { id, settings: _ } = self.get_block(*choice) {
        choices_strings.push(id.clone());
      }
    }

    choices_strings
  }

  fn update_choices(&mut self) {
    self.choices = Vec::default();

    if self.block_stack.is_empty() {
      return;
    }

    let last_block_id = self.block_stack.last().unwrap();
    let last_block = self.get_block(*last_block_id).clone();

    let settings = last_block.get_settings();

    for child in &settings.children {
      if *child < self.database.blocks.len() {
        match self.get_block(*child) {
          cuentitos_common::Block::Choice { id: _, settings: _ } => {
            if self.meets_requirements(*child) {
              self.choices.push(*child)
            }
          }
          cuentitos_common::Block::Bucket { name: _, settings } => {
            if let Some(picked_block) = self.get_random_block_from_bucket(&settings.clone()) {
              if let cuentitos_common::Block::Choice { id: _, settings: _ } =
                self.get_block(picked_block)
              {
                self.choices.push(picked_block);
              }
            }
          }
          _ => {}
        }
      }
    }
  }
}

#[cfg(test)]
mod test {

  use std::{collections::HashMap, fmt::Display, str::FromStr, vec};

  use crate::Runtime;
  use cuentitos_common::{
    Block, BlockSettings, Chance, Condition, Config, Database, FrequencyModifier, Modifier,
    Requirement, SectionKey, VariableKind,
  };

  #[test]
  fn new_runtime_works_correctly() {
    let database = Database {
      blocks: vec![Block::default()],
      sections: HashMap::default(),
      config: Config::default(),
    };
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database);
  }

  #[test]
  fn jump_to_section_works_correctly() {
    let section_1 = Block::Section {
      id: "section_1".to_string(),
      settings: BlockSettings {
        children: vec![3],
        ..Default::default()
      },
    };
    let section_2 = Block::Section {
      id: "section_1".to_string(),
      settings: BlockSettings::default(),
    };
    let subsection = Block::Subsection {
      id: "subsection".to_string(),
      settings: BlockSettings {
        children: vec![4],
        ..Default::default()
      },
    };
    let text_1 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let mut sections: HashMap<SectionKey, usize> = HashMap::default();
    sections.insert(
      SectionKey {
        section: "section_1".to_string(),
        subsection: None,
      },
      0,
    );
    sections.insert(
      SectionKey {
        section: "section_2".to_string(),
        subsection: None,
      },
      1,
    );
    sections.insert(
      SectionKey {
        section: "section_2".to_string(),
        subsection: Some("subsection".to_string()),
      },
      2,
    );
    let database = Database {
      blocks: vec![section_1, section_2, subsection, text_1, text_2],
      sections,
      config: Config::default(),
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };
    runtime.jump_to_section("section_2".to_string(), Some("subsection".to_string()));
    runtime.update_stack();
    assert_eq!(runtime.block_stack, vec![2, 4]);
    runtime.jump_to_section("section_1".to_string(), None);
    runtime.update_stack();
    assert_eq!(runtime.block_stack, vec![0, 3]);
  }

  #[test]
  fn get_choices_works_correctly() {
    let settings = BlockSettings {
      children: vec![1, 2, 3],
      ..Default::default()
    };

    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let child_text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1, choice_2, child_text],
      sections: HashMap::default(),
      config: Config::default(),
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };

    runtime.update_choices();
    let expected_result = vec![1, 2];
    assert_eq!(runtime.choices, expected_result);
  }
  #[test]
  fn get_choices_strings_works_correctly() {
    let settings = BlockSettings {
      children: vec![1, 2, 3],
      ..Default::default()
    };

    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "a".to_string(),
      settings: BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "b".to_string(),
      settings: BlockSettings::default(),
    };

    let child_text = Block::Text {
      id: "c".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1, choice_2, child_text],
      sections: HashMap::default(),
      config: Config::default(),
    };
    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };
    let choices = runtime.get_choices_strings();
    let expected_result = vec!["a".to_string(), "b".to_string()];
    assert_eq!(choices, expected_result);
  }
  #[test]
  fn updates_stack_to_first_child_correctly() {
    let settings = BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };
    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let child_1 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let child_2 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), child_1.clone(), child_2.clone()],
      sections: HashMap::default(),
      config: Config::default(),
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };
    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), 1);
  }

  #[test]
  fn update_stack_to_next_sibling_correctly() {
    let settings = BlockSettings {
      children: vec![2, 3, 4],
      ..Default::default()
    };

    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let sibling = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let child_1 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let child_2 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let child_3 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![
        parent.clone(),
        sibling.clone(),
        child_1.clone(),
        child_2.clone(),
        child_3.clone(),
      ],
      sections: HashMap::default(),
      config: Config::default(),
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0, 2],
      ..Default::default()
    };

    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), 3);
    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), 4);
    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), 1);
  }

  #[test]
  fn get_next_block_output_works_correctly() {
    let settings = BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };
    let parent = Block::Text {
      id: "parent".to_string(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "1".to_string(),
      settings: BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "2".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1.clone(), choice_2],
      sections: HashMap::default(),
      config: Config::default(),
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };

    let output = runtime.get_next_block_output();
    let expected_output = Some(crate::Block {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
    });

    assert_eq!(output, expected_output);
  }

  #[test]
  fn next_block_works_correctly() {
    let settings = BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };

    let parent = Block::Text {
      id: "parent".to_string(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "1".to_string(),
      settings: BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "2".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1.clone(), choice_2.clone()],
      sections: HashMap::default(),
      config: Config::default(),
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };

    let output = runtime.next_block();
    let expected_output = Some(crate::Block {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
    });

    assert_eq!(output, expected_output);
    assert_eq!(runtime.block_stack, vec![0]);
  }

  #[test]
  fn next_output_doesnt_work_with_empty_file() {
    let mut runtime = Runtime::new(Database::default());
    assert_eq!(runtime.next_block(), None);
  }

  #[test]
  fn get_random_block_from_bucket_works_correctly() {
    let settings = BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };

    let bucket = Block::Bucket {
      name: None,
      settings,
    };

    let settings = BlockSettings {
      chance: cuentitos_common::Chance::Frequency(50),
      ..Default::default()
    };

    let text_1 = Block::Text {
      id: String::default(),
      settings,
    };

    let settings = BlockSettings {
      chance: cuentitos_common::Chance::Frequency(50),
      ..Default::default()
    };

    let text_2 = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![bucket, text_1, text_2],
      sections: HashMap::default(),
      config: Config::default(),
    };
    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };

    runtime.set_seed(2);

    let bucket_settings = runtime.get_block(0).get_settings().clone();
    let id = runtime
      .get_random_block_from_bucket(&bucket_settings)
      .unwrap();
    assert_eq!(id, 1);
    runtime.push_stack(1);
    let bucket_settings = runtime.get_block(0).get_settings().clone();
    let id = runtime
      .get_random_block_from_bucket(&bucket_settings)
      .unwrap();
    assert_eq!(id, 2);
  }

  #[test]
  fn int_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let database = Database {
      blocks: Vec::default(),
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);

    let current_health: i32 = runtime.get_variable("health").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_health, expected_value);

    runtime.set_variable("health", 100).unwrap();
    let current_health: i32 = runtime.get_variable("health").unwrap();
    assert_eq!(current_health, 100);
  }

  #[test]
  fn integer_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "health".to_string(),
      added_value: "100".to_string(),
      is_override: false,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_health: i32 = runtime.get_variable("health").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_health, expected_value);
    runtime.apply_modifiers();
    let current_health: i32 = runtime.get_variable("health").unwrap();
    assert_eq!(current_health, expected_value + 100);
    runtime.apply_modifiers();
    let current_health: i32 = runtime.get_variable("health").unwrap();
    assert_eq!(current_health, expected_value + 200);
  }

  #[test]
  fn float_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "speed".to_string(),
      added_value: "1.5".to_string(),
      is_override: false,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_speed, expected_value);
    runtime.apply_modifiers();
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    assert_eq!(current_speed, expected_value + 1.5);
    runtime.apply_modifiers();
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    assert_eq!(current_speed, expected_value + 3.);
  }

  #[test]
  fn integer_override_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "health".to_string(),
      added_value: "100".to_string(),
      is_override: true,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_health: i32 = runtime.get_variable("health").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_health, expected_value);
    runtime.apply_modifiers();
    let current_health: i32 = runtime.get_variable("health").unwrap();
    assert_eq!(current_health, 100);
    runtime.apply_modifiers();
    let current_health: i32 = runtime.get_variable("health").unwrap();
    assert_eq!(current_health, 100);
  }

  #[test]
  fn float_override_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "speed".to_string(),
      added_value: "1.5".to_string(),
      is_override: true,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_speed, expected_value);
    runtime.apply_modifiers();
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    assert_eq!(current_speed, 1.5);
    runtime.apply_modifiers();
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    assert_eq!(current_speed, 1.5);
  }

  #[test]
  fn bool_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "bike".to_string(),
      added_value: "true".to_string(),
      is_override: false,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_bike: bool = runtime.get_variable("bike").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_bike, expected_value);
    runtime.apply_modifiers();
    let current_bike: bool = runtime.get_variable("bike").unwrap();
    assert_eq!(current_bike, true);
  }

  #[test]
  fn string_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "message".to_string(),
      added_value: "hello".to_string(),
      is_override: false,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_message: String = runtime.get_variable("message").unwrap();
    let expected_value = variable_kind.get_default_value();
    assert_eq!(current_message, expected_value);

    runtime.apply_modifiers();
    let current_message: String = runtime.get_variable("message").unwrap();
    assert_eq!(current_message, "hello".to_string());
  }

  #[test]
  fn enum_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config { variables };

    let modifier = Modifier {
      variable: "time_of_day".to_string(),
      added_value: "Night".to_string(),
      is_override: false,
    };
    let settings = BlockSettings {
      modifiers: vec![modifier],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];

    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_time_of_day, expected_value);

    runtime.apply_modifiers();
    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    assert_eq!(current_time_of_day, TimeOfDay::Night);
  }

  #[test]
  fn float_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let database = Database {
      blocks: Vec::default(),
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);

    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_speed, expected_value);

    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    assert_eq!(current_speed, 1.5);
  }

  #[test]
  fn bool_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config { variables };

    let database = Database {
      blocks: Vec::default(),
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);

    let current_bike: bool = runtime.get_variable("bike").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_bike, expected_value);

    runtime.set_variable("bike", true).unwrap();
    let current_speed: bool = runtime.get_variable("bike").unwrap();
    assert_eq!(current_speed, true);
  }

  #[test]
  fn string_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config { variables };

    let database = Database {
      blocks: Vec::default(),
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);

    let current_message: String = runtime.get_variable("message").unwrap();
    let expected_value = variable_kind.get_default_value();
    assert_eq!(current_message, expected_value);

    runtime
      .set_variable("message", "hello".to_string())
      .unwrap();
    let current_message: String = runtime.get_variable("message").unwrap();
    assert_eq!(current_message, "hello".to_string());
  }

  #[test]
  fn enum_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config { variables };

    let database = Database {
      blocks: Vec::default(),
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);

    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_time_of_day, expected_value);

    runtime
      .set_variable("time_of_day", TimeOfDay::Night)
      .unwrap();
    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    assert_eq!(current_time_of_day, TimeOfDay::Night);
  }

  #[test]
  fn integer_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: cuentitos_common::Operator::Equal,
        value: "100".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }

  #[test]
  fn integer_greater_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: cuentitos_common::Operator::GreaterOrEqualThan,
        value: "100".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn integer_greater_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: cuentitos_common::Operator::GreaterThan,
        value: "100".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn integer_less_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: cuentitos_common::Operator::LessOrEqualThan,
        value: "100".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }
  #[test]
  fn integer_less_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: cuentitos_common::Operator::LessThan,
        value: "100".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }
  #[test]
  fn integer_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: cuentitos_common::Operator::NotEqual,
        value: "100".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }
  #[test]
  fn float_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: cuentitos_common::Operator::Equal,
        value: "1.5".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }

  #[test]
  fn float_greater_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: cuentitos_common::Operator::GreaterOrEqualThan,
        value: "1.5".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn float_greater_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: cuentitos_common::Operator::GreaterThan,
        value: "1.5".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn float_less_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: cuentitos_common::Operator::LessOrEqualThan,
        value: "1.5".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }
  #[test]
  fn float_less_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: cuentitos_common::Operator::LessThan,
        value: "1.5".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }
  #[test]
  fn float_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: cuentitos_common::Operator::NotEqual,
        value: "1.5".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn bool_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "bike".to_string(),
        operator: cuentitos_common::Operator::Equal,
        value: "true".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime.set_variable("bike", true).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn bool_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "bike".to_string(),
        operator: cuentitos_common::Operator::NotEqual,
        value: "true".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime.set_variable("bike", true).unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }

  #[test]
  fn string_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "message".to_string(),
        operator: cuentitos_common::Operator::Equal,
        value: "hello".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime
      .set_variable("message", "hello".to_string())
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn string_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "message".to_string(),
        operator: cuentitos_common::Operator::NotEqual,
        value: "hello".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime
      .set_variable("message", "hello".to_string())
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }

  #[test]
  fn enum_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "time_of_day".to_string(),
        operator: cuentitos_common::Operator::Equal,
        value: "Night".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
    runtime
      .set_variable("time_of_day", TimeOfDay::Night)
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
  }

  #[test]
  fn enum_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config { variables };

    let requirement = Requirement {
      condition: Condition {
        variable: "time_of_day".to_string(),
        operator: cuentitos_common::Operator::NotEqual,
        value: "Night".to_string(),
      },
    };
    let settings = BlockSettings {
      requirements: vec![requirement],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0);
    assert!(meets_requirement);
    runtime
      .set_variable("time_of_day", TimeOfDay::Night)
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0);
    assert!(!meets_requirement);
  }

  #[test]
  fn frequency_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config { variables };

    let freq_mod = FrequencyModifier {
      condition: Condition {
        variable: "bike".to_string(),
        operator: cuentitos_common::Operator::Equal,
        value: "true".to_string(),
      },
      value: -100,
    };
    let settings = BlockSettings {
      frequency_modifiers: vec![freq_mod],
      chance: Chance::Frequency(100),
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings: settings.clone(),
    };

    let database = Database {
      blocks: vec![block],
      sections: HashMap::default(),
      config,
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let frequency_with_modifier = runtime.get_frequency_with_modifier(&settings);
    assert_eq!(frequency_with_modifier, 100);
    runtime.set_variable("bike", true).unwrap();
    let frequency_with_modifier = runtime.get_frequency_with_modifier(&settings);
    assert_eq!(frequency_with_modifier, 0);
  }
  #[derive(Debug, Default, PartialEq, Eq)]
  enum TimeOfDay {
    #[default]
    Day,
    Night,
  }

  #[derive(Debug, PartialEq, Eq)]
  struct TestError;

  impl FromStr for TimeOfDay {
    type Err = TestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      match s {
        "Day" => Ok(TimeOfDay::Day),
        "Night" => Ok(TimeOfDay::Night),
        _ => Err(TestError),
      }
    }
  }
  impl Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
    }
  }
}
