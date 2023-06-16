use std::println;

use cuentitos_common::BlockId;
use cuentitos_common::BlockSettings;
use cuentitos_common::Database;
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
}

impl Runtime {
  pub fn new(database: Database) -> Runtime {
    Runtime {
      database,
      ..Default::default()
    }
  }

  pub fn set_seed(&mut self, seed: u64) {
    self.seed = seed;
    self.rng = Some(Pcg32::seed_from_u64(seed));
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
        cuentitos_common::Block::Section {
          id: _,
          settings: _,
          subsections: _,
        } => {
          self.update_stack();
        }
        cuentitos_common::Block::Subsection {
          id: _,
          settings: _,
          subsections: _,
        } => {
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
      cuentitos_common::NextBlock::Section(_) => todo!(),
    }

    self.block_stack.pop();

    if self.block_stack.is_empty() {
      self.block_stack.push(last_block_id + 1);
      let last_block_id = *self.block_stack.last().unwrap();
      let last_block = self.get_block(last_block_id).clone();
      match last_block {
        cuentitos_common::Block::Section {
          id: _,
          settings: _,
          subsections: _,
        } => {
          self.update_stack();
        }
        cuentitos_common::Block::Subsection {
          id: _,
          settings: _,
          subsections: _,
        } => {
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

  use std::vec;

  use crate::Runtime;
  use cuentitos_common::{Block, BlockSettings, Database};

  #[test]
  fn new_runtime_works_correctly() {
    let database = Database {
      blocks: vec![Block::default()],
    };
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database);
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
