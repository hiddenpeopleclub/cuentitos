use std::fmt::Display;

use crate::GameState;
use crate::RuntimeError;
use cuentitos_common::condition::ComparisonOperator;
use cuentitos_common::modifier::ModifierOperator;
use cuentitos_common::BlockId;
use cuentitos_common::BlockSettings;
use cuentitos_common::Condition;
use cuentitos_common::Database;
use cuentitos_common::Function;
use cuentitos_common::LanguageId;
use cuentitos_common::Modifier;
use cuentitos_common::NextBlock;
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
  pub tags: Vec<String>,
  pub functions: Vec<Function>,
}

pub type DivertData = SectionKey;

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
  pub current_locale: LanguageId,
}

impl Runtime {
  pub fn new(database: Database) -> Runtime {
    let game_state: GameState = GameState::from_config(&database.config);
    let current_locale = database.i18n.default_locale.clone();
    Runtime {
      database,
      game_state,
      current_locale,
      ..Default::default()
    }
  }

  pub fn reset(&mut self) {
    self.block_stack.clear();
    self.choices.clear();
    self.game_state = GameState::from_config(&self.database.config);
    self.set_seed(self.seed);
  }

  pub fn set_locale<T>(&mut self, locale: T) -> Result<(), String>
  where
    T: AsRef<str>,
  {
    let locale = locale.as_ref().to_string();
    if self.database.i18n.has_locale(&locale) {
      self.current_locale = locale;
      Ok(())
    } else {
      Err("Missing Locale".to_string())
    }
  }

  pub fn set_seed(&mut self, seed: u64) {
    self.seed = seed;
    self.rng = Some(Pcg32::seed_from_u64(seed));
  }

  pub fn divert(&mut self, divert: &DivertData) -> Result<(), RuntimeError> {
    let next = self.get_section_block_id(divert)?;
    self.push_stack(next)
  }

  fn get_section_block_id(&mut self, divert: &DivertData) -> Result<BlockId, RuntimeError> {
    let current_section = self.game_state.current_section.clone();
    if divert.subsection.is_none() {
      if let Some(current_section) = current_section {
        let key = SectionKey {
          section: current_section,
          subsection: Some(divert.section.clone()),
        };
        if let Some(block_id) = self.database.sections.get(&key) {
          return Ok(*block_id);
        }
      }
    }

    if let Some(block_id) = self.database.sections.get(divert) {
      Ok(*block_id)
    } else {
      Err(RuntimeError::SectionDoesntExist(divert.clone()))
    }
  }

  pub fn next_block(&mut self) -> Result<Block, RuntimeError> {
    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    self.update_stack()?;

    while !self.next_block_meets_requirements()? {
      self.pop_stack_and_find_next()?;
    }
    self.current_block()
  }

  pub fn get_block(&self, id: BlockId) -> &cuentitos_common::Block {
    &self.database.blocks[id]
  }

  pub fn current_block(&mut self) -> Result<Block, RuntimeError> {
    let id = match self.block_stack.last() {
      Some(id) => id,
      None => return Err(RuntimeError::EmptyStack),
    };

    let block = self.get_block(*id);
    let settings = block.get_settings();
    let tags = settings.tags.clone();
    let functions = settings.functions.clone();
    match block {
      cuentitos_common::Block::Text { id, settings: _ } => Ok(Block {
        text: self.database.i18n.get_translation(&self.current_locale, id),
        choices: self.get_choices_strings(),
        tags,
        functions,
      }),
      cuentitos_common::Block::Choice { id: _, settings: _ } => {
        Err(RuntimeError::UnexpectedBlock {
          expected_block: "text".to_string(),
          block_found: "choice".to_string(),
        })
      }
      cuentitos_common::Block::Bucket {
        name: _,
        settings: _,
      } => Err(RuntimeError::UnexpectedBlock {
        expected_block: "text".to_string(),
        block_found: "bucket".to_string(),
      }),
      cuentitos_common::Block::Section { id: _, settings: _ } => {
        Err(RuntimeError::UnexpectedBlock {
          expected_block: "text".to_string(),
          block_found: "section".to_string(),
        })
      }
      cuentitos_common::Block::Subsection { id: _, settings: _ } => {
        Err(RuntimeError::UnexpectedBlock {
          expected_block: "text".to_string(),
          block_found: "subsection".to_string(),
        })
      }
      cuentitos_common::Block::Divert {
        next: _,
        settings: _,
      } => Err(RuntimeError::UnexpectedBlock {
        expected_block: "text".to_string(),
        block_found: "divert".to_string(),
      }),
    }
  }

  pub fn pick_choice(&mut self, choice: usize) -> Result<Block, RuntimeError> {
    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    let choices = &self.choices;

    if choices.is_empty() {
      return Err(RuntimeError::NoChoices);
    }

    if choice >= choices.len() {
      return Err(RuntimeError::InvalidChoice {
        total_choices: choices.len(),
        choice_picked: choice,
      });
    }

    if choices[choice] >= self.database.blocks.len() {
      return Err(RuntimeError::InvalidBlockId(choices[choice]));
    }

    self.push_stack(choices[choice])?;
    self.next_block()
  }

  pub fn set_variable<R, T>(&mut self, variable: R, value: T) -> Result<(), RuntimeError>
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
        return Err(RuntimeError::UnsupportedVariableType {
          type_found: t.to_string(),
        });
      }
    } else {
      return Err(RuntimeError::VariableDoesntExist(variable));
    }
    Ok(())
  }

  pub fn get_variable_kind<R>(&self, variable: R) -> Result<VariableKind, RuntimeError>
  where
    R: AsRef<str>,
  {
    let variable = variable.as_ref();

    if self.database.config.variables.contains_key(variable) {
      Ok(self.database.config.variables[variable].clone())
    } else {
      Err(RuntimeError::VariableDoesntExist(variable.to_string()))
    }
  }

  pub fn get_variable<R, T>(&self, variable: R) -> Result<T, RuntimeError>
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
        match value.parse::<T>() {
          Ok(value) => Ok(value),
          Err(_) => Err(RuntimeError::UnknownParsingError),
        }
      } else {
        Err(RuntimeError::UnsupportedVariableType {
          type_found: t.to_string(),
        })
      }
    } else {
      Err(RuntimeError::VariableDoesntExist(variable))
    }
  }

  pub fn apply_modifier(&mut self, modifier: &Modifier) -> Result<(), RuntimeError> {
    match self.get_variable_kind(&modifier.variable)? {
      VariableKind::Integer => {
        let value = &modifier.value.parse::<i32>();
        match value {
          Ok(value) => self.apply_integer_modifier(&modifier.variable, *value, &modifier.operator),
          Err(e) => Err(RuntimeError::ParseIntError(e.clone())),
        }
      }
      VariableKind::Float => {
        let value = &modifier.value.parse::<f32>();
        match value {
          Ok(value) => self.apply_float_modifier(&modifier.variable, *value, &modifier.operator),
          Err(e) => Err(RuntimeError::ParseFloatError(e.clone())),
        }
      }
      VariableKind::Bool => {
        let value = &modifier.value.parse::<bool>();
        match value {
          Ok(value) => self.set_variable(&modifier.variable, *value),
          Err(e) => Err(RuntimeError::ParseBoolError(e.clone())),
        }
      }
      _ => self.set_variable(&modifier.variable, modifier.value.clone()),
    }
  }

  pub fn get_choices_strings(&self) -> Vec<String> {
    let mut choices_strings = Vec::default();
    for choice in &self.choices {
      if let cuentitos_common::Block::Choice { id, settings: _ } = self.get_block(*choice) {
        choices_strings.push(self.database.i18n.get_translation(&self.current_locale, id));
      }
    }

    choices_strings
  }

  fn next_block_meets_requirements(&mut self) -> Result<bool, RuntimeError> {
    if let Some(id) = self.block_stack.last() {
      self.meets_requirements(*id)
    } else {
      Ok(false)
    }
  }

  fn meets_requirements(&mut self, id: BlockId) -> Result<bool, RuntimeError> {
    for requirement in &self.get_block(id).get_settings().requirements {
      if !self.meets_condition(&requirement.condition)? {
        return Ok(false);
      }
    }
    Ok(self.roll_chances_for_block(id))
  }

  fn meets_condition(&self, condition: &Condition) -> Result<bool, RuntimeError> {
    let kind = self.get_variable_kind(condition.variable.clone())?;

    match kind {
      VariableKind::Integer => {
        if let Ok(current_value) = self.get_variable::<&str, i32>(&condition.variable) {
          if let Ok(condition_value) = condition.value.parse::<i32>() {
            match condition.operator {
              ComparisonOperator::Equal => return Ok(current_value == condition_value),
              ComparisonOperator::NotEqual => return Ok(current_value != condition_value),
              ComparisonOperator::GreaterThan => return Ok(current_value > condition_value),
              ComparisonOperator::LessThan => return Ok(current_value < condition_value),
              ComparisonOperator::GreaterOrEqualThan => {
                return Ok(current_value >= condition_value)
              }
              ComparisonOperator::LessOrEqualThan => return Ok(current_value <= condition_value),
            }
          }
        }
      }
      VariableKind::Float => {
        if let Ok(current_value) = self.get_variable::<&str, f32>(&condition.variable) {
          if let Ok(condition_value) = condition.value.parse::<f32>() {
            match condition.operator {
              ComparisonOperator::Equal => return Ok(current_value == condition_value),
              ComparisonOperator::NotEqual => return Ok(current_value != condition_value),
              ComparisonOperator::GreaterThan => return Ok(current_value > condition_value),
              ComparisonOperator::LessThan => return Ok(current_value < condition_value),
              ComparisonOperator::GreaterOrEqualThan => {
                return Ok(current_value >= condition_value)
              }
              ComparisonOperator::LessOrEqualThan => return Ok(current_value <= condition_value),
            }
          }
        }
      }
      VariableKind::Bool => {
        if let Ok(current_value) = self.get_variable::<&str, bool>(&condition.variable) {
          if let Ok(condition_value) = condition.value.parse::<bool>() {
            match condition.operator {
              ComparisonOperator::Equal => return Ok(current_value == condition_value),
              ComparisonOperator::NotEqual => return Ok(current_value != condition_value),
              _ => {}
            }
          }
        }
      }
      _ => {
        if let Ok(current_value) = self.get_variable::<&str, String>(&condition.variable) {
          if let Ok(condition_value) = condition.value.parse::<String>() {
            match condition.operator {
              ComparisonOperator::Equal => return Ok(current_value == condition_value),
              ComparisonOperator::NotEqual => return Ok(current_value != condition_value),
              _ => {}
            }
          }
        }
      }
    }

    Ok(false)
  }

  fn roll_chances_for_block(&mut self, id: BlockId) -> bool {
    match self.get_block(id).get_settings().chance {
      cuentitos_common::Chance::None => true,
      cuentitos_common::Chance::Frequency(_) => true,
      cuentitos_common::Chance::Probability(probability) => self.random_float() < probability,
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

  fn random_float(&mut self) -> f32 {
    let mut rng = match &self.rng {
      Some(rng) => rng.clone(),
      None => Pcg32::from_entropy(),
    };
    let num = rng.gen();

    self.rng = Some(rng);
    num
  }

  fn random_with_max(&mut self, max: u32) -> u32 {
    let mut rng = match &self.rng {
      Some(rng) => rng.clone(),
      None => Pcg32::from_entropy(),
    };

    let num = rng.gen_range(0..max);
    self.rng = Some(rng);
    num
  }

  fn get_frequency_with_modifier(&self, settings: &BlockSettings) -> Result<u32, RuntimeError> {
    match settings.chance {
      cuentitos_common::Chance::None => Ok(0),
      cuentitos_common::Chance::Frequency(frequency) => {
        let mut final_frequency = frequency as i32;
        for freq_mod in &settings.frequency_modifiers {
          if self.meets_condition(&freq_mod.condition)? {
            final_frequency += freq_mod.value;
          }
        }
        Ok(final_frequency as u32)
      }
      cuentitos_common::Chance::Probability(_) => {
        Err(RuntimeError::FrequencyModifierWithProbability)
      }
    }
  }

  fn apply_integer_modifier(
    &mut self,
    variable: &String,
    value: i32,
    operator: &ModifierOperator,
  ) -> Result<(), RuntimeError> {
    let previous_value = self.get_variable::<&str, i32>(variable);
    match previous_value {
      Ok(previous_value) => match operator {
        ModifierOperator::Set => self.set_variable(variable, value),
        ModifierOperator::Add => self.set_variable(variable, previous_value + value),
        ModifierOperator::Substract => self.set_variable(variable, previous_value - value),
        ModifierOperator::Multiply => self.set_variable(variable, previous_value * value),
        ModifierOperator::Divide => self.set_variable(variable, previous_value / value),
      },
      Err(e) => Err(e),
    }
  }

  fn apply_float_modifier(
    &mut self,
    variable: &String,
    value: f32,
    operator: &ModifierOperator,
  ) -> Result<(), RuntimeError> {
    let previous_value = self.get_variable::<&str, f32>(variable);
    match previous_value {
      Ok(previous_value) => match operator {
        ModifierOperator::Set => self.set_variable(variable, value),
        ModifierOperator::Add => self.set_variable(variable, previous_value + value),
        ModifierOperator::Substract => self.set_variable(variable, previous_value - value),
        ModifierOperator::Multiply => self.set_variable(variable, previous_value * value),
        ModifierOperator::Divide => self.set_variable(variable, previous_value / value),
      },
      Err(e) => Err(e),
    }
  }

  fn apply_modifiers(&mut self) -> Result<(), RuntimeError> {
    let id = match self.block_stack.last() {
      Some(id) => id,
      None => return Err(RuntimeError::EmptyStack),
    };
    let block = self.get_block(*id);
    for modifier in block.get_settings().modifiers.clone() {
      self.apply_modifier(&modifier)?;
    }
    Ok(())
  }

  fn push_stack(&mut self, id: BlockId) -> Result<(), RuntimeError> {
    self.block_stack.push(id);

    if self.get_block(id).get_settings().unique {
      if self.game_state.uniques_played.contains(&id) {
        return self.pop_stack_and_find_next();
      } else {
        self.game_state.uniques_played.push(id);
      }
    }

    self.apply_modifiers()?;

    let block = self.get_block(id).clone();
    match block {
      cuentitos_common::Block::Section { id, settings: _ } => {
        self.game_state.current_section = Some(id);
        self.game_state.current_subsection = None;
        self.update_stack()
      }
      cuentitos_common::Block::Subsection { id, settings: _ } => {
        self.game_state.current_subsection = Some(id);
        self.update_stack()
      }
      cuentitos_common::Block::Text { id: _, settings: _ } => {
        self.update_choices()?;
        Ok(())
      }
      cuentitos_common::Block::Choice { id: _, settings: _ } => {
        self.update_choices()?;
        Ok(())
      }
      cuentitos_common::Block::Bucket {
        name: _,
        settings: _,
      } => {
        if let Some(next) = self.get_random_block_from_bucket(block.get_settings())? {
          self.push_stack(next)
        } else {
          self.update_choices()
        }
      }
      cuentitos_common::Block::Divert { next, settings: _ } => {
        let next = self.get_divert_next(next)?;
        self.push_stack(next)
      }
    }
  }

  fn get_divert_next(&mut self, next: NextBlock) -> Result<BlockId, RuntimeError> {
    match next {
      NextBlock::BlockId(id) => Ok(id),
      NextBlock::EndOfFile => {
        self.reset();
        Err(RuntimeError::StoryFinished)
      }
      NextBlock::Section(section_key) => self.get_section_block_id(&section_key),
    }
  }
  fn update_stack(&mut self) -> Result<(), RuntimeError> {
    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    if self.block_stack.is_empty() {
      return self.push_stack(0);
    }

    let last_block_id = match self.block_stack.last() {
      Some(id) => id,
      None => return Err(RuntimeError::EmptyStack),
    };

    if last_block_id >= &self.database.blocks.len() {
      return Err(RuntimeError::InvalidBlockId(*last_block_id));
    }

    let settings = self.get_block(*last_block_id).get_settings().clone();

    if !settings.children.is_empty() {
      if let Some(child) = self.get_next_child_in_stack(&settings, 0)? {
        return self.push_stack(child);
      }
    }

    self.pop_stack_and_find_next()
  }

  fn get_next_child_in_stack(
    &mut self,
    settings: &BlockSettings,
    next_child: usize,
  ) -> Result<Option<BlockId>, RuntimeError> {
    if next_child >= settings.children.len() {
      return Ok(None);
    }

    let id = settings.children[next_child];
    match self.get_block(id) {
      cuentitos_common::Block::Choice { id: _, settings: _ } => {
        if self.choices.contains(&id) {
          Err(RuntimeError::WaitingForChoice(self.get_choices_strings()))
        } else {
          self.get_next_child_in_stack(settings, next_child + 1)
        }
      }
      cuentitos_common::Block::Section { id, settings: _ } => {
        Err(RuntimeError::SectionAtLowerLevel(id.clone()))
      }
      cuentitos_common::Block::Subsection { id, settings: _ } => {
        Err(RuntimeError::SectionAtLowerLevel(id.clone()))
      }
      _ => Ok(Some(settings.children[0])),
    }
  }

  fn pop_stack_and_find_next(&mut self) -> Result<(), RuntimeError> {
    let last_block_id: usize = match self.block_stack.last() {
      Some(id) => *id,
      None => return Err(RuntimeError::EmptyStack),
    };

    self.block_stack.pop();
    if self.block_stack.is_empty() {
      return self.push_stack(last_block_id + 1);
    }

    let new_block_id: &usize = match self.block_stack.last() {
      Some(id) => id,
      None => return Err(RuntimeError::EmptyStack),
    };

    let parent = &self.database.blocks[*new_block_id].clone();

    let parent_settings = parent.get_settings();
    let mut previous_block_found = false;
    for sibling in &parent_settings.children {
      if previous_block_found {
        if let cuentitos_common::Block::Choice { id: _, settings: _ } = self.get_block(*sibling) {
          continue;
        }
        return self.push_stack(*sibling);
      }
      if *sibling == last_block_id {
        previous_block_found = true;
      }
    }

    self.pop_stack_and_find_next()
  }

  fn get_random_block_from_bucket(
    &mut self,
    settings: &BlockSettings,
  ) -> Result<Option<BlockId>, RuntimeError> {
    let mut total_frequency = 0;
    for child in &settings.children {
      if self.meets_requirements(*child)? {
        let child_settings = self.get_block(*child).get_settings();
        let frequency = self.get_frequency_with_modifier(child_settings)?;
        total_frequency += frequency;
      }
    }

    let mut random_number = self.random_with_max(total_frequency);

    for child in &settings.children {
      if self.meets_requirements(*child)? {
        let child_settings = self.get_block(*child).get_settings();
        let frequency = self.get_frequency_with_modifier(child_settings)?;
        if random_number <= frequency {
          return Ok(Some(*child));
        }
        random_number -= frequency;
      }
    }
    Ok(None)
  }

  fn update_choices(&mut self) -> Result<(), RuntimeError> {
    self.choices = Vec::default();

    if self.block_stack.is_empty() {
      return Err(RuntimeError::EmptyStack);
    }

    let last_block_id: &usize = match self.block_stack.last() {
      Some(id) => id,
      None => return Err(RuntimeError::EmptyStack),
    };

    let last_block = self.get_block(*last_block_id).clone();

    let settings = last_block.get_settings();

    for child in &settings.children {
      if *child < self.database.blocks.len() {
        match self.get_block(*child) {
          cuentitos_common::Block::Choice { id: _, settings: _ } => {
            if self.meets_requirements(*child)? {
              self.choices.push(*child)
            }
          }
          cuentitos_common::Block::Bucket { name: _, settings } => {
            if let Some(picked_block) = self.get_random_block_from_bucket(&settings.clone())? {
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

    Ok(())
  }
}

#[cfg(test)]
mod test {

  use std::{collections::HashMap, fmt::Display, str::FromStr, vec};

  use crate::{runtime::DivertData, Runtime, RuntimeError};
  use cuentitos_common::{
    condition::ComparisonOperator, modifier::ModifierOperator, Block, BlockSettings, Chance,
    Condition, Config, Database, FrequencyModifier, Function, I18n, LanguageDb, LanguageId,
    Modifier, NextBlock, Requirement, SectionKey, VariableKind,
  };

  #[test]
  fn new_runtime_works_correctly() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database);
  }

  #[test]
  fn divert_works_correctly() {
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
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };
    runtime
      .divert(&DivertData {
        section: "section_2".to_string(),
        subsection: Some("subsection".to_string()),
      })
      .unwrap();
    assert_eq!(runtime.block_stack, vec![2, 4]);
    runtime
      .divert(&DivertData {
        section: "section_1".to_string(),
        subsection: None,
      })
      .unwrap();
    assert_eq!(runtime.block_stack, vec![2, 4, 0, 3]);
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
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };

    runtime.update_choices().unwrap();
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

    let mut en: LanguageDb = HashMap::default();
    en.insert("a".to_string(), "a".to_string());
    en.insert("b".to_string(), "b".to_string());
    en.insert("c".to_string(), "c".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1, choice_2, child_text],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      current_locale: "en".to_string(),
      ..Default::default()
    };
    runtime.update_choices().unwrap();
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
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      ..Default::default()
    };
    runtime.update_stack().unwrap();
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
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0, 2],
      ..Default::default()
    };

    runtime.update_stack().unwrap();
    assert_eq!(*runtime.block_stack.last().unwrap(), 3);
    runtime.update_stack().unwrap();
    assert_eq!(*runtime.block_stack.last().unwrap(), 4);
    runtime.update_stack().unwrap();
    assert_eq!(*runtime.block_stack.last().unwrap(), 1);
  }

  #[test]
  fn current_block_works_correctly() {
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

    let mut en: LanguageDb = HashMap::default();
    en.insert("1".to_string(), "1".to_string());
    en.insert("2".to_string(), "2".to_string());
    en.insert("parent".to_string(), "parent".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1.clone(), choice_2],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      block_stack: vec![0],
      current_locale: "en".to_string(),
      ..Default::default()
    };

    runtime.update_choices().unwrap();
    let output = runtime.current_block().unwrap();
    let expected_output = crate::Block {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
      ..Default::default()
    };

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

    let mut en: LanguageDb = HashMap::default();
    en.insert("1".to_string(), "1".to_string());
    en.insert("2".to_string(), "2".to_string());
    en.insert("parent".to_string(), "parent".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1.clone(), choice_2.clone()],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output = runtime.next_block().unwrap();
    let expected_output = crate::Block {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
    assert_eq!(runtime.block_stack, vec![0]);
  }
  /*
  #[test]
  fn next_output_doesnt_work_with_empty_file() {
    let mut runtime = Runtime::new(Database::default());
    assert_eq!(runtime.next_block(), None);
  } */

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
      ..Default::default()
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
      .unwrap()
      .unwrap();
    assert_eq!(id, 1);
    runtime.push_stack(1).unwrap();
    let bucket_settings = runtime.get_block(0).get_settings().clone();
    let id = runtime
      .get_random_block_from_bucket(&bucket_settings)
      .unwrap()
      .unwrap();
    assert_eq!(id, 2);
  }

  #[test]
  fn int_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let database = Database {
      blocks: Vec::default(),
      sections: HashMap::default(),
      config,
      ..Default::default()
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
    let config = Config {
      variables,
      ..Default::default()
    };

    let modifier_1 = Modifier {
      variable: "health".to_string(),
      value: "100".to_string(),
      operator: ModifierOperator::Set,
    };
    let modifier_2 = Modifier {
      variable: "health".to_string(),
      value: "50".to_string(),
      operator: ModifierOperator::Substract,
    };

    let modifier_3 = Modifier {
      variable: "health".to_string(),
      value: "3".to_string(),
      operator: ModifierOperator::Multiply,
    };

    let modifier_4 = Modifier {
      variable: "health".to_string(),
      value: "2".to_string(),
      operator: ModifierOperator::Divide,
    };

    let settings = BlockSettings {
      modifiers: vec![modifier_1, modifier_2, modifier_3, modifier_4],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_health: i32 = runtime.get_variable("health").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_health, expected_value);
    runtime.apply_modifiers().unwrap();
    let current_health: i32 = runtime.get_variable("health").unwrap();
    assert_eq!(current_health, 75);
  }

  #[test]
  fn float_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let modifier_1 = Modifier {
      variable: "speed".to_string(),
      value: "1".to_string(),
      operator: ModifierOperator::Set,
    };
    let modifier_2 = Modifier {
      variable: "speed".to_string(),
      value: "0.5".to_string(),
      operator: ModifierOperator::Substract,
    };

    let modifier_3 = Modifier {
      variable: "speed".to_string(),
      value: "3".to_string(),
      operator: ModifierOperator::Multiply,
    };

    let modifier_4 = Modifier {
      variable: "speed".to_string(),
      value: "2".to_string(),
      operator: ModifierOperator::Divide,
    };

    let settings = BlockSettings {
      modifiers: vec![modifier_1, modifier_2, modifier_3, modifier_4],
      ..Default::default()
    };
    let block = Block::Text {
      id: String::default(),
      settings,
    };

    let database = Database {
      blocks: vec![block],
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_speed, expected_value);
    runtime.apply_modifiers().unwrap();
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    assert_eq!(current_speed, 0.75);
  }

  #[test]
  fn bool_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let modifier = Modifier {
      variable: "bike".to_string(),
      value: "true".to_string(),
      ..Default::default()
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_bike: bool = runtime.get_variable("bike").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_bike, expected_value);
    runtime.apply_modifiers().unwrap();
    let current_bike: bool = runtime.get_variable("bike").unwrap();
    assert_eq!(current_bike, true);
  }

  #[test]
  fn string_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let modifier = Modifier {
      variable: "message".to_string(),
      value: "hello".to_string(),
      ..Default::default()
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let current_message: String = runtime.get_variable("message").unwrap();
    let expected_value = variable_kind.get_default_value();
    assert_eq!(current_message, expected_value);

    runtime.apply_modifiers().unwrap();
    let current_message: String = runtime.get_variable("message").unwrap();
    assert_eq!(current_message, "hello".to_string());
  }

  #[test]
  fn enum_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let modifier = Modifier {
      variable: "time_of_day".to_string(),
      value: "Night".to_string(),
      ..Default::default()
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];

    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_time_of_day, expected_value);

    runtime.apply_modifiers().unwrap();
    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    assert_eq!(current_time_of_day, TimeOfDay::Night);
  }

  #[test]
  fn float_variables_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let database = Database {
      blocks: Vec::default(),
      config,
      ..Default::default()
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
    let config = Config {
      variables,
      ..Default::default()
    };

    let database = Database {
      blocks: Vec::default(),
      config,
      ..Default::default()
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
    let config = Config {
      variables,
      ..Default::default()
    };

    let database = Database {
      blocks: Vec::default(),
      config,
      ..Default::default()
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
    let config = Config {
      variables,
      ..Default::default()
    };

    let database = Database {
      blocks: Vec::default(),
      config,
      ..Default::default()
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
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: ComparisonOperator::Equal,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }

  #[test]
  fn integer_greater_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: ComparisonOperator::GreaterOrEqualThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn integer_greater_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: ComparisonOperator::GreaterThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn integer_less_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: ComparisonOperator::LessOrEqualThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }
  #[test]
  fn integer_less_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: ComparisonOperator::LessThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }
  #[test]
  fn integer_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Integer;
    variables.insert("health".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "health".to_string(),
        operator: ComparisonOperator::NotEqual,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("health", 100).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("health", 150).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }
  #[test]
  fn float_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: ComparisonOperator::Equal,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }

  #[test]
  fn float_greater_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: ComparisonOperator::GreaterOrEqualThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn float_greater_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: ComparisonOperator::GreaterThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn float_less_or_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: ComparisonOperator::LessOrEqualThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }
  #[test]
  fn float_less_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: ComparisonOperator::LessThan,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }
  #[test]
  fn float_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Float;
    variables.insert("speed".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "speed".to_string(),
        operator: ComparisonOperator::NotEqual,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("speed", 1.5 as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("speed", 2. as f32).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn bool_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "bike".to_string(),
        operator: ComparisonOperator::Equal,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime.set_variable("bike", true).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn bool_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "bike".to_string(),
        operator: ComparisonOperator::NotEqual,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime.set_variable("bike", true).unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }

  #[test]
  fn string_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "message".to_string(),
        operator: ComparisonOperator::Equal,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime
      .set_variable("message", "hello".to_string())
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn string_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::String;
    variables.insert("message".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "message".to_string(),
        operator: ComparisonOperator::NotEqual,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime
      .set_variable("message", "hello".to_string())
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }

  #[test]
  fn enum_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "time_of_day".to_string(),
        operator: ComparisonOperator::Equal,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
    runtime
      .set_variable("time_of_day", TimeOfDay::Night)
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
  }

  #[test]
  fn enum_not_equal_requirements_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Enum(vec!["Day".to_string(), "Night".to_string()]);
    variables.insert("time_of_day".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let requirement = Requirement {
      condition: Condition {
        variable: "time_of_day".to_string(),
        operator: ComparisonOperator::NotEqual,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(meets_requirement);
    runtime
      .set_variable("time_of_day", TimeOfDay::Night)
      .unwrap();
    let meets_requirement = runtime.meets_requirements(0).unwrap();
    assert!(!meets_requirement);
  }

  #[test]
  fn frequency_modifiers_work() {
    let mut variables = HashMap::default();

    let variable_kind = VariableKind::Bool;
    variables.insert("bike".to_string(), variable_kind.clone());
    let config = Config {
      variables,
      ..Default::default()
    };

    let freq_mod = FrequencyModifier {
      condition: Condition {
        variable: "bike".to_string(),
        operator: ComparisonOperator::Equal,
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
      config,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.block_stack = vec![0];
    let frequency_with_modifier = runtime.get_frequency_with_modifier(&settings).unwrap();
    assert_eq!(frequency_with_modifier, 100);
    runtime.set_variable("bike", true).unwrap();
    let frequency_with_modifier = runtime.get_frequency_with_modifier(&settings).unwrap();
    assert_eq!(frequency_with_modifier, 0);
  }

  #[test]
  fn unique_only_plays_once() {
    let section = Block::Section {
      id: "section".to_string(),
      settings: BlockSettings {
        children: vec![1, 2, 3],
        ..Default::default()
      },
    };

    let mut sections: HashMap<SectionKey, usize> = HashMap::default();
    sections.insert(
      SectionKey {
        section: "section".to_string(),
        subsection: None,
      },
      0,
    );

    let settings = BlockSettings {
      unique: true,
      ..Default::default()
    };
    let text_1 = Block::Text {
      id: String::default(),
      settings,
    };

    let text_2 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let divert = Block::Divert {
      next: NextBlock::Section(SectionKey {
        section: "section".to_string(),
        subsection: None,
      }),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![section, text_1, text_2, divert],
      sections,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    runtime.update_stack().unwrap();
    assert_eq!(1, *runtime.block_stack.last().unwrap());
    runtime.update_stack().unwrap();
    assert_eq!(2, *runtime.block_stack.last().unwrap());
    runtime.update_stack().unwrap();
    assert_eq!(2, *runtime.block_stack.last().unwrap());
  }

  #[test]
  fn tags_work() {
    let tags = vec!["a_tag".to_string()];
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings {
        tags: tags.clone(),
        ..Default::default()
      },
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let output_tags = runtime.next_block().unwrap().tags;
    assert_eq!(tags, output_tags);
  }

  #[test]
  fn functions_work() {
    let functions = vec![Function {
      name: "a_function".to_string(),
      parameters: vec!["parameter".to_string()],
    }];

    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings {
        functions: functions.clone(),
        ..Default::default()
      },
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let output_functions = runtime.next_block().unwrap().functions;
    assert_eq!(functions, output_functions);
  }

  #[test]
  fn invalid_id_in_stack_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.block_stack.push(1);
    let err = runtime.update_stack().unwrap_err();
    assert_eq!(err, RuntimeError::InvalidBlockId(1));
  }

  #[test]
  fn invalid_id_in_choice_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.choices.push(1);
    let err = runtime.pick_choice(0).unwrap_err();
    assert_eq!(err, RuntimeError::InvalidBlockId(1));
  }

  #[test]
  fn next_block_throws_error_if_theres_a_choice() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings {
        children: vec![2],
        ..Default::default()
      },
    };

    let choice = Block::Choice {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let text_2 = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text, text_2, choice],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.update_stack().unwrap();
    let err = runtime.update_stack().unwrap_err();
    assert_eq!(
      err,
      RuntimeError::WaitingForChoice(vec!["MISSING LOCALE ``".to_string()])
    );
  }

  #[test]
  fn section_at_lower_level_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings {
        children: vec![1],
        ..Default::default()
      },
    };

    let section = Block::Section {
      id: "section".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text, section],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.update_stack().unwrap();
    let err = runtime.update_stack().unwrap_err();
    assert_eq!(
      err,
      RuntimeError::SectionAtLowerLevel("section".to_string())
    );
  }
  #[test]
  fn throws_error_when_story_finishes() {
    let text: Block = Block::Divert {
      next: NextBlock::EndOfFile,
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = runtime.update_stack().unwrap_err();
    assert_eq!(err, RuntimeError::StoryFinished);
  }

  #[test]
  fn divert_throws_error_if_section_doesnt_exist() {
    let section_key = SectionKey {
      section: "section".to_string(),
      subsection: Some("subsection".to_string()),
    };
    let divert = Block::Divert {
      next: NextBlock::Section(section_key.clone()),
      settings: BlockSettings::default(),
    };
    let database = Database {
      blocks: vec![divert],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = runtime.divert(&section_key.clone()).unwrap_err();
    assert_eq!(err, RuntimeError::SectionDoesntExist(section_key));
  }

  #[test]
  fn current_block_can_only_be_text() {
    let choice = Block::Choice {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![choice],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.block_stack.push(0);
    let err = runtime.current_block().unwrap_err();

    assert_eq!(
      err,
      RuntimeError::UnexpectedBlock {
        expected_block: "text".to_string(),
        block_found: "choice".to_string()
      }
    );
  }

  #[test]
  fn current_block_throws_error_on_empty_stack() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = runtime.current_block().unwrap_err();
    assert_eq!(err, RuntimeError::EmptyStack);
  }

  #[test]
  fn apply_modifiers_throws_error_on_empty_stack() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = runtime.apply_modifiers().unwrap_err();
    assert_eq!(err, RuntimeError::EmptyStack);
  }

  #[test]
  fn empty_database_throws_error() {
    let database = Database {
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = runtime.next_block().unwrap_err();
    assert_eq!(err, RuntimeError::EmptyDatabase);
    let err = runtime.update_stack().unwrap_err();
    assert_eq!(err, RuntimeError::EmptyDatabase);
    let err = runtime.pick_choice(0).unwrap_err();
    assert_eq!(err, RuntimeError::EmptyDatabase);
  }

  #[test]
  fn picking_choice_when_no_options_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);

    let err = runtime.pick_choice(0).unwrap_err();
    assert_eq!(err, RuntimeError::NoChoices);
  }

  #[test]
  fn picking_invalid_choice_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.choices.push(0);
    let err = runtime.pick_choice(1).unwrap_err();
    assert_eq!(
      err,
      RuntimeError::InvalidChoice {
        total_choices: 1,
        choice_picked: 1
      }
    );
  }

  #[test]
  fn setting_unsupported_variable_type_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Integer);

    let database = Database {
      blocks: vec![text],
      config,
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);

    let err = runtime
      .set_variable("variable", UnsupportedType::default())
      .unwrap_err();
    assert_eq!(
      err,
      RuntimeError::UnsupportedVariableType {
        type_found: "cuentitos_runtime::runtime::test::UnsupportedType".to_string()
      }
    );
  }

  #[test]
  fn getting_unsupported_variable_type_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Integer);

    let database = Database {
      blocks: vec![text],
      config,
      ..Default::default()
    };
    let runtime = Runtime::new(database);

    let err = runtime
      .get_variable::<&str, UnsupportedType>("variable")
      .unwrap_err();
    assert_eq!(
      err,
      RuntimeError::UnsupportedVariableType {
        type_found: "cuentitos_runtime::runtime::test::UnsupportedType".to_string()
      }
    );
  }

  #[test]
  fn setting_non_existent_variable_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);

    let err = runtime.set_variable("variable", 0).unwrap_err();
    assert_eq!(
      err,
      RuntimeError::VariableDoesntExist("variable".to_string())
    );
  }

  #[test]
  fn getting_non_existent_variable_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let runtime = Runtime::new(database);

    let err = runtime.get_variable::<&str, i32>("variable").unwrap_err();
    assert_eq!(
      err,
      RuntimeError::VariableDoesntExist("variable".to_string())
    );
  }

  #[test]
  fn getting_non_existent_variable_kind_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let runtime = Runtime::new(database);

    let err: RuntimeError = runtime.get_variable_kind("variable").unwrap_err();
    assert_eq!(
      err,
      RuntimeError::VariableDoesntExist("variable".to_string())
    );
  }

  #[derive(Debug, Default, PartialEq, Eq)]
  enum TimeOfDay {
    #[default]
    Day,
    Night,
  }

  #[derive(Debug, PartialEq, Eq)]
  struct TestError;

  #[derive(Default, Debug)]
  struct UnsupportedType;

  impl Display for UnsupportedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "")
    }
  }

  impl FromStr for UnsupportedType {
    type Err = TestError;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
      Ok(UnsupportedType::default())
    }
  }

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
