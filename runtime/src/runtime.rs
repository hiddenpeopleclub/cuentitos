use std::fmt::Display;
use std::println;

use crate::GameState;
use cuentitos_common::BlockId;
use cuentitos_common::BlockSettings;
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
  rng: Option<Pcg32>,
  seed: u64,
  game_state: GameState,
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
    let key = SectionKey {
      section,
      subsection,
    };
    if let Some(block_id) = self.database.sections.get(&key) {
      self.block_stack.clear();
      self.block_stack.push(*block_id);
    } else {
      println!("Can't find section: {:?}", key);
    }
  }

  pub fn next_block(&mut self) -> Option<Block> {
    if self.database.blocks.is_empty() {
      return None;
    }

    self.update_stack();

    while !self.roll_chances_for_next_block() {
      self.update_stack();
    }

    self.get_next_block_output()
  }

  pub fn roll_chances_for_block(&mut self, id: BlockId) -> bool {
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

  pub fn roll_chances_for_next_block(&mut self) -> bool {
    if let Some(last_block) = self.block_stack.last() {
      return self.roll_chances_for_block(*last_block);
    }
    false
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

    self.block_stack.push(choices[choice]);
    self.next_block()
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
        || (t == "alloc::string::String"
          && self.database.config.variables[&variable] == VariableKind::String)
        || (t == "&str" && self.database.config.variables[&variable] == VariableKind::String)
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
        || (t == "alloc::string::String"
          && self.database.config.variables[&variable] == VariableKind::String)
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

    Err("Invalid Variable".to_string())
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

  fn update_stack(&mut self) {
    if self.block_stack.is_empty() {
      self.block_stack.push(0);
      let last_block_id = *self.block_stack.last().unwrap();
      let last_block = self.get_block(last_block_id).clone();
      match last_block {
        cuentitos_common::Block::Section { id: _, settings: _ } => {
          self.update_stack();
        }
        cuentitos_common::Block::Subsection { id: _, settings: _ } => {
          self.update_stack();
        }
        _ => {}
      }
      return;
    }

    let last_block_id = self.block_stack.last().unwrap();

    if last_block_id >= &self.database.blocks.len() {
      return;
    }

    let last_block = self.get_block(*last_block_id);

    let settings = last_block.get_settings();
    if !settings.children.is_empty() {
      match self.get_block(settings.children[0]) {
        cuentitos_common::Block::Text { id: _, settings: _ } => {
          self.block_stack.push(settings.children[0]);
          return;
        }
        cuentitos_common::Block::Choice { id: _, settings: _ } => {
          println!("Make a choice\n");
          return;
        }
        cuentitos_common::Block::Bucket { name: _, settings } => {
          if let Some(new_block) = self.get_random_block_from_bucket(&settings.clone()) {
            self.block_stack.push(new_block);
            return;
          }
        }
        _ => {}
      }
    }

    self.pop_stack_and_find_next();
  }

  fn get_random_block_from_bucket(&mut self, settings: &BlockSettings) -> Option<BlockId> {
    let mut total_frequency = 0;
    for child in &settings.children {
      if let cuentitos_common::Chance::Frequency(frequency) =
        self.get_block(*child).get_settings().chance
      {
        total_frequency += frequency;
      }
    }

    //TODO remove unwrap
    let mut random_number = self.random_with_max(total_frequency).unwrap();

    for child in &settings.children {
      if let cuentitos_common::Chance::Frequency(frequency) =
        self.get_block(*child).get_settings().chance
      {
        if random_number <= frequency {
          return Some(*child);
        }
        random_number -= frequency;
      }
    }
    None
  }

  fn pop_stack_and_find_next(&mut self) {
    let last_block_id = *self.block_stack.last().unwrap();

    let last_block = self.get_block(last_block_id).clone();

    let last_settings = last_block.get_settings();
    match &last_settings.next {
      cuentitos_common::NextBlock::None => {}
      cuentitos_common::NextBlock::BlockId(other_id) => {
        self.block_stack.pop();
        self.block_stack.push(*other_id);
        return;
      }
      cuentitos_common::NextBlock::EndOfFile => {
        println!("Story finished\n");
        self.block_stack = vec![0];
        return;
      }
      cuentitos_common::NextBlock::Section(key) => {
        self.jump_to_section(key.section.clone(), key.subsection.clone());
        self.update_stack();
      }
    }

    self.block_stack.pop();

    if self.block_stack.is_empty() {
      self.block_stack.push(last_block_id + 1);
      let last_block_id = *self.block_stack.last().unwrap();
      let last_block = self.get_block(last_block_id).clone();
      match last_block {
        cuentitos_common::Block::Section { id: _, settings: _ } => {
          self.update_stack();
        }
        cuentitos_common::Block::Subsection { id: _, settings: _ } => {
          self.update_stack();
        }
        _ => {}
      }
      return;
    }

    let parent = &self.database.blocks[*self.block_stack.last().unwrap()].clone();
    let parent_settings = parent.get_settings();
    let mut child_found = false;
    for child in &parent_settings.children {
      if child_found {
        match self.get_block(*child).clone() {
          cuentitos_common::Block::Text { id: _, settings: _ } => self.block_stack.push(*child),
          cuentitos_common::Block::Choice { id: _, settings: _ } => {
            continue;
          }
          cuentitos_common::Block::Bucket { name: _, settings } => {
            if let Some(new_block) = self.get_random_block_from_bucket(&settings.clone()) {
              self.block_stack.push(new_block);
              return;
            }
          }
          _ => {
            continue;
          }
        }
        if let cuentitos_common::Block::Choice { id: _, settings: _ } = self.get_block(*child) {
          continue;
        }
        self.block_stack.push(*child);
        return;
      }
      if *child == last_block_id {
        child_found = true;
      }
    }

    self.pop_stack_and_find_next()
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
            if self.roll_chances_for_block(*child) {
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

  use std::{collections::HashMap, vec};

  use crate::Runtime;
  use cuentitos_common::{Block, BlockSettings, Config, Database, SectionKey, VariableKind};

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
    runtime.block_stack.push(1);
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

    runtime.set_variable("message", "hello").unwrap();
    let current_message: String = runtime.get_variable("message").unwrap();
    assert_eq!(current_message, "hello".to_string());
  }
  /*
    #[test]
    fn enum_variables_work()
    {
      let mut variables  = HashMap::default();

      let variable_kind = VariableKind::Enum { values: vec!["Day".to_string(),"Night".to_string()] };
      variables.insert("time_of_day".to_string(), variable_kind.clone());
      let config = Config{
        variables
      };

      let database = Database {
        blocks: Vec::default(),
        sections: HashMap::default(),
        config,
      };

      let mut runtime = Runtime::new(database);

      let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
      let expected_value = variable_kind.get_default_value().parse().unwrap();
      assert_eq!(current_time_of_day, expected_value);

      runtime.set_variable("time_of_day", TimeOfDay::Night).unwrap();
      let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
      assert_eq!(current_time_of_day, TimeOfDay::Night);

      #[derive(Debug,Default,PartialEq, Eq)]
      enum TimeOfDay{
        #[default]
        Day,
        Night
      }

      #[derive(Debug, PartialEq, Eq)]
      struct TestError;

      impl FromStr for TimeOfDay
      {
          type Err = TestError;

          fn from_str(s: &str) -> Result<Self, Self::Err> {
              match s
              {
                "Day" => Ok(TimeOfDay::Day),
                "Night" => Ok(TimeOfDay::Night),
                _ => Err(TestError)
              }
          }
      }
      impl Display for TimeOfDay
      {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}",self)
          }
      }
    }
  */

  #[test]
  fn roll_chances_for_next_block_works_correctly() {
    let settings = BlockSettings {
      ..Default::default()
    };

    let text_with_no_chances = Block::Text {
      id: String::default(),
      settings,
    };

    let settings = BlockSettings {
      chance: cuentitos_common::Chance::Probability(1.),
      ..Default::default()
    };

    let text_with_100_chances = Block::Text {
      id: String::default(),
      settings,
    };

    let settings = BlockSettings {
      chance: cuentitos_common::Chance::Probability(0.),
      ..Default::default()
    };

    let text_with_0_chances = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![
        text_with_no_chances,
        text_with_100_chances,
        text_with_0_chances,
      ],
      sections: HashMap::default(),
      config: Config::default(),
    };
    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };

    assert_eq!(runtime.roll_chances_for_next_block(), true);
    runtime.update_stack();
    assert_eq!(runtime.roll_chances_for_next_block(), true);
    runtime.update_stack();
    assert_eq!(runtime.roll_chances_for_next_block(), false);
  }
}
