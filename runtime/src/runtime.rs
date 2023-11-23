use std::fmt::Display;

use crate::GameState;
use crate::RuntimeError;
use cuentitos_common::condition::ComparisonOperator;
use cuentitos_common::modifier::ModifierOperator;
use cuentitos_common::BlockId;
use cuentitos_common::Condition;
use cuentitos_common::Database;
use cuentitos_common::Function;
use cuentitos_common::LanguageId;
use cuentitos_common::Modifier;
use cuentitos_common::NextBlock;
use cuentitos_common::Script;
use cuentitos_common::Section;
use cuentitos_common::VariableKind;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

type BucketName = String;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct BlockStackData {
  pub id: BlockId,
  pub chance: Chance,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Output {
  pub text: String,
  pub text_ids: Vec<String>,
  pub choices: Vec<String>,
  pub choices_ids: Vec<String>,
  pub blocks: Vec<Block>,
}

impl Output {
  pub fn from_blocks(blocks: Vec<Block>, runtime: &Runtime) -> Result<Self, RuntimeError> {
    if let Some(last_block) = blocks.last() {
      match runtime.get_cuentitos_block(last_block.get_settings().id)? {
        cuentitos_common::Block::Text { id, settings: _ } => Ok(Output {
          text_ids: vec![id.clone()],
          text: runtime
            .database
            .i18n
            .get_translation(&runtime.current_locale, id),
          choices: runtime.get_current_choices_strings()?,
          choices_ids: runtime.get_current_choices_ids()?,
          blocks,
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
        cuentitos_common::Block::Divert {
          next: _,
          settings: _,
        } => Err(RuntimeError::UnexpectedBlock {
          expected_block: "text".to_string(),
          block_found: "divert".to_string(),
        }),
        cuentitos_common::Block::BoomerangDivert {
          next: _,
          settings: _,
        } => Err(RuntimeError::UnexpectedBlock {
          expected_block: "text".to_string(),
          block_found: "boomerang divert".to_string(),
        }),
      }
    } else {
      Err(RuntimeError::EmptyStack)
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Block {
  Text {
    text: String,
    settings: BlockSettings,
  },
  Choice {
    text: String,
    settings: BlockSettings,
  },
  Bucket {
    name: Option<BucketName>,
    settings: BlockSettings,
  },
  Section {
    settings: BlockSettings,
  },
  Divert {
    next: NextBlock,
    settings: BlockSettings,
  },
  BoomerangDivert {
    next: NextBlock,
    settings: BlockSettings,
  },
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum Chance {
  #[default]
  None,
  Probability(f32),
  Frequency {
    value: u32,
    total_frequency: u32,
  },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct BlockSettings {
  pub id: BlockId,
  pub script: Script,
  pub chance: Chance,
  pub tags: Vec<String>,
  pub functions: Vec<Function>,
  pub changed_variables: Vec<String>,
  pub section: Option<Section>,
}

impl Block {
  pub fn get_settings(&self) -> &BlockSettings {
    match self {
      Block::Text { text: _, settings } => settings,
      Block::Choice { text: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section { settings } => settings,
      Block::Divert { next: _, settings } => settings,
      Block::BoomerangDivert { next: _, settings } => settings,
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Runtime {
  pub database: Database,
  #[serde(skip)]
  pub game_state: GameState,
  #[serde(skip)]
  rng: Option<Pcg32>,
  seed: u64,
  pub current_locale: LanguageId,
  #[serde(skip)]
  pub history: Vec<GameState>,
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

  pub fn reset_story(&mut self) {
    self.set_seed(self.seed);
    self.game_state.block_stack.clear();
    self.game_state.section = None;
    self.history.clear();
    self.game_state.choices.clear();
  }
  pub fn reset_state(&mut self) {
    self.game_state = GameState::from_config(&self.database.config);
  }
  pub fn reset_all(&mut self) {
    self.reset_story();
    self.reset_state();
  }

  pub fn set_locale<T>(&mut self, locale: T) -> Result<(), RuntimeError>
  where
    T: AsRef<str>,
  {
    let locale = locale.as_ref().to_string();
    if self.database.i18n.has_locale(&locale) {
      self.current_locale = locale;
      Ok(())
    } else {
      Err(RuntimeError::MissingLocale(locale))
    }
  }

  pub fn get_current_section(&self) -> &Option<Section> {
    &self.game_state.section
  }

  pub fn get_current_block_stack(&self) -> &Vec<BlockStackData> {
    &self.game_state.block_stack
  }

  pub fn get_current_choices(&self) -> &Vec<usize> {
    &self.game_state.choices
  }

  pub fn set_seed(&mut self, seed: u64) {
    self.seed = seed;
    self.rng = Some(Pcg32::seed_from_u64(seed));
  }

  pub fn divert(&mut self, section: &Section) -> Result<Vec<Block>, RuntimeError> {
    let history_entry = self.game_state.clone();

    let new_stack: Vec<usize> = self.get_section_block_ids(section)?;

    self.game_state.block_stack.clear();
    let mut blocks_added = Vec::default();
    for block in new_stack {
      if !self.meets_requirements(block)? {
        self.game_state.block_stack.push(BlockStackData {
          id: block,
          chance: Chance::None,
        });
        blocks_added.append(&mut self.pop_stack_and_push_next()?);
        self.add_to_history(history_entry);
        return Ok(blocks_added);
      }
      blocks_added.push(Self::push_stack(self, block)?);
    }

    self.add_to_history(history_entry);
    Ok(blocks_added)
  }

  pub fn boomerang_divert(&mut self, section: &Section) -> Result<Vec<Block>, RuntimeError> {
    let history_entry = self.game_state.clone();
    let new_stack: Vec<usize> = self.get_section_block_ids(section)?;

    let mut blocks_added = Vec::default();
    for block in new_stack {
      if !self.meets_requirements(block)? {
        self.game_state.block_stack.push(BlockStackData {
          id: block,
          chance: Chance::None,
        });

        blocks_added.append(&mut self.pop_stack_and_push_next()?);
        self.add_to_history(history_entry);
        return Ok(blocks_added);
      }
      blocks_added.push(Self::push_stack(self, block)?);
    }
    self.add_to_history(history_entry);
    Ok(blocks_added)
  }

  pub fn peek_next(&self) -> Result<Output, RuntimeError> {
    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    let mut peek_runtime = self.clone();
    let blocks = peek_runtime.update_stack()?;
    Output::from_blocks(blocks, self)
  }

  pub fn next_block(&mut self) -> Result<Output, RuntimeError> {
    let history_entry = self.game_state.clone();

    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    let blocks = Self::update_stack(self)?;

    self.add_to_history(history_entry);

    Output::from_blocks(blocks, self)
  }

  pub fn progress_story(&mut self) -> Result<Output, RuntimeError> {
    match self.database.config.story_progress_style {
      cuentitos_common::StoryProgressStyle::Next => self.next_block(),
      cuentitos_common::StoryProgressStyle::Skip => self.skip(),
    }
  }

  pub fn skip(&mut self) -> Result<Output, RuntimeError> {
    let mut output = self.next_block()?;

    while output.choices.is_empty() {
      let pre_self = self.clone();
      let mut new_output = match self.next_block() {
        Ok(output) => output,
        Err(err) => match err {
          RuntimeError::StoryFinished => {
            *self = pre_self;
            break;
          }
          _ => return Err(err),
        },
      };
      output.blocks.append(&mut new_output.blocks);
      output.text_ids.append(&mut new_output.text_ids);
      output.choices_ids.append(&mut new_output.choices_ids);
      output.choices = new_output.choices;
      output.text += "\n";
      output.text += &new_output.text;
    }

    Ok(output)
  }

  pub fn skip_all(&mut self) -> Result<Output, RuntimeError> {
    let mut output = self.next_block()?;

    while output.choices.is_empty() {
      let pre_self = self.clone();
      let mut new_output = match self.next_block() {
        Ok(output) => output,
        Err(err) => match err {
          RuntimeError::StoryFinished => {
            *self = pre_self;
            break;
          }
          _ => return Err(err),
        },
      };
      output.blocks.append(&mut new_output.blocks);
      output.choices_ids.append(&mut new_output.choices_ids);
      if let Some(text_id) = new_output.text_ids.last() {
        output.text_ids = vec![text_id.clone()];
      }
      output.choices = new_output.choices;
      output.text = new_output.text;
    }

    Ok(output)
  }

  pub fn get_block(&self, stack_data: &BlockStackData) -> Result<Block, RuntimeError> {
    let id = stack_data.id;
    let cuentitos_block = self.get_cuentitos_block(id)?;
    let cuentitos_settings = cuentitos_block.get_settings();
    let script = cuentitos_settings.script.clone();
    let tags = cuentitos_settings.tags.clone();
    let functions = cuentitos_settings.functions.clone();
    let chance = stack_data.chance.clone();
    let changed_variables = self.get_changed_variables(cuentitos_settings)?;
    let section = cuentitos_settings.section.clone();

    let settings = BlockSettings {
      id,
      script,
      tags,
      functions,
      chance,
      changed_variables,
      section,
    };

    let block = match cuentitos_block {
      cuentitos_common::Block::Text { id, settings: _ } => Block::Text {
        text: self.database.i18n.get_translation(&self.current_locale, id),
        settings,
      },
      cuentitos_common::Block::Choice { id, settings: _ } => Block::Choice {
        text: self.database.i18n.get_translation(&self.current_locale, id),
        settings,
      },
      cuentitos_common::Block::Bucket { name, settings: _ } => Block::Bucket {
        name: name.clone(),
        settings,
      },
      cuentitos_common::Block::Section { id: _, settings: _ } => Block::Section { settings },
      cuentitos_common::Block::Divert { next, settings: _ } => Block::Divert {
        next: next.clone(),
        settings,
      },
      cuentitos_common::Block::BoomerangDivert { next, settings: _ } => Block::BoomerangDivert {
        next: next.clone(),
        settings,
      },
    };

    Ok(block)
  }

  pub fn current(&self) -> Result<Output, RuntimeError> {
    let stack_data = match self.game_state.block_stack.last() {
      Some(id) => id,
      None => return Err(RuntimeError::EmptyStack),
    };
    let blocks = vec![self.get_block(stack_data)?];
    Output::from_blocks(blocks, self)
  }

  pub fn pick_choice(&mut self, choice: usize) -> Result<Output, RuntimeError> {
    let history_entry = self.game_state.clone();

    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    if self.game_state.choices.is_empty() {
      return Err(RuntimeError::NoChoices);
    }

    if choice >= self.game_state.choices.len() {
      return Err(RuntimeError::InvalidChoice {
        total_choices: self.game_state.choices.len(),
        choice_picked: choice,
      });
    }

    if self.game_state.choices[choice] >= self.database.blocks.len() {
      return Err(RuntimeError::InvalidBlockId(
        self.game_state.choices[choice],
      ));
    }

    self.add_to_history(history_entry);

    let mut blocks = self.push_stack_until_text_or_choice(self.game_state.choices[choice])?;
    let mut output = self.progress_story()?;
    blocks.append(&mut output.blocks);
    output.blocks = blocks;

    Ok(output)
  }

  pub fn rewind(&mut self) -> Result<(), RuntimeError> {
    if let Some(previous_state) = self.history.pop() {
      self.game_state = previous_state;
      Ok(())
    } else {
      Err(RuntimeError::RewindWithNoHistory())
    }
  }

  pub fn rewind_to_choice(&mut self) -> Result<(), RuntimeError> {
    if self.game_state.block_stack.is_empty() {
      return Ok(());
    }

    if let Some(previous_state) = self.history.pop() {
      self.game_state = previous_state;
      if self.game_state.choices.is_empty() {
        self.rewind_to_choice()
      } else {
        Ok(())
      }
    } else {
      Err(RuntimeError::RewindWithNoHistory())
    }
  }

  pub fn rewind_to(&mut self, index: usize) -> Result<(), RuntimeError> {
    if !self.database.config.keep_history {
      return Err(RuntimeError::RewindWithNoHistory());
    }

    if index >= self.history.len() {
      return Err(RuntimeError::RewindWithToInvalidIndex {
        index,
        current_index: self.history.len(),
      });
    }

    while self.history.len() > index {
      self.rewind()?;
    }

    Ok(())
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
        || t == "alloc::string::String"
        || t == "&str"
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

  pub fn get_current_choices_strings(&self) -> Result<Vec<String>, RuntimeError> {
    let mut choices_strings = Vec::default();
    for choice in &self.game_state.choices {
      if let cuentitos_common::Block::Choice { id, settings: _ } =
        self.get_cuentitos_block(*choice)?
      {
        choices_strings.push(self.database.i18n.get_translation(&self.current_locale, id));
      }
    }

    Ok(choices_strings)
  }

  pub fn get_current_choices_ids(&self) -> Result<Vec<String>, RuntimeError> {
    let mut choices_id = Vec::default();
    for choice in &self.game_state.choices {
      if let cuentitos_common::Block::Choice { id, settings: _ } =
        self.get_cuentitos_block(*choice)?
      {
        choices_id.push(id.clone());
      }
    }

    Ok(choices_id)
  }

  fn add_to_history(&mut self, game_state: GameState) {
    if self.database.config.keep_history {
      self.history.push(game_state);
    }
  }

  fn get_cuentitos_block(&self, id: BlockId) -> Result<&cuentitos_common::Block, RuntimeError> {
    if id < self.database.blocks.len() {
      Ok(&self.database.blocks[id])
    } else {
      Err(RuntimeError::InvalidBlockId(id))
    }
  }

  fn get_changed_variables(
    &self,
    settings: &cuentitos_common::BlockSettings,
  ) -> Result<Vec<String>, RuntimeError> {
    let mut variables = Vec::default();
    for modifier in &settings.modifiers {
      let variable = modifier.variable.clone();
      if !variables.contains(&variable) {
        variables.push(variable);
      }
    }

    Ok(variables)
  }

  fn get_section_block_ids_recursive(
    &self,
    target_section: &Section,
    ids: &mut Vec<BlockId>,
  ) -> Result<(), RuntimeError> {
    match self.database.sections.get(target_section) {
      Some(id) => {
        if let Some(current_section) = &self.game_state.section {
          if current_section.is_child_of(target_section) {
            return Ok(());
          }
        }
        if let Some(parent) = &target_section.parent {
          self.get_section_block_ids_recursive(parent, ids)?;
        }
        ids.push(*id);
        Ok(())
      }
      None => Err(RuntimeError::SectionDoesntExist(target_section.clone())),
    }
  }

  fn get_actual_section(&self, section: &Section) -> Result<Section, RuntimeError> {
    fn append_to_parent(value: Section, into: &mut Section) {
      match &mut into.parent {
        Some(parent) => append_to_parent(value, parent),
        None => {
          into.parent = Some(Box::new(value));
        }
      }
    }

    let mut section_mut = section.clone();

    let section_exists = match self.database.sections.contains_key(&section_mut) {
      true => true,
      false => match &self.game_state.section {
        Some(current_section) => {
          append_to_parent(current_section.clone(), &mut section_mut);
          match self.database.sections.contains_key(&section_mut) {
            true => true,
            false => {
              section_mut.parent = section_mut.parent.unwrap().parent;
              self.database.sections.contains_key(&section_mut)
            }
          }
        }
        None => false,
      },
    };

    match section_exists {
      true => Ok(section_mut),
      false => Err(RuntimeError::SectionDoesntExist(section.clone())),
    }
  }
  fn get_section_block_ids(
    &mut self,
    target_section: &Section,
  ) -> Result<Vec<BlockId>, RuntimeError> {
    let target_section = self.get_actual_section(target_section)?;
    if let Some(current_section) = &self.game_state.section {
      if current_section.is_child_of(&target_section) {
        if let Some(id) = self.database.sections.get(&target_section) {
          return Ok(vec![*id]);
        }
      }
    }

    let mut ids: Vec<usize> = Vec::default();
    self.get_section_block_ids_recursive(&target_section, &mut ids)?;

    Ok(ids)
  }

  fn meets_requirements_and_chance(&mut self, id: BlockId) -> Result<bool, RuntimeError> {
    if self.meets_requirements(id)? {
      Ok(self.roll_chances_for_block(id)?)
    } else {
      Ok(false)
    }
  }

  fn meets_requirements(&self, id: BlockId) -> Result<bool, RuntimeError> {
    let settings = self.get_cuentitos_block(id)?.get_settings();

    if settings.unique && self.game_state.uniques_played.contains(&id) {
      return Ok(false);
    }

    for requirement in &settings.requirements {
      if !self.meets_condition(&requirement.condition)? {
        return Ok(false);
      }
    }
    Ok(true)
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

  fn roll_chances_for_block(&mut self, id: BlockId) -> Result<bool, RuntimeError> {
    match self.get_cuentitos_block(id)?.get_settings().chance {
      cuentitos_common::Chance::None => Ok(true),
      cuentitos_common::Chance::Frequency(_) => Ok(true),
      cuentitos_common::Chance::Probability(probability) => Ok(self.random_float() < probability),
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

  fn get_frequency_with_modifier(
    &self,
    settings: &cuentitos_common::BlockSettings,
  ) -> Result<u32, RuntimeError> {
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

  fn apply_modifiers(&mut self, modifiers: &Vec<Modifier>) -> Result<(), RuntimeError> {
    for modifier in modifiers {
      self.apply_modifier(modifier)?;
    }
    Ok(())
  }

  fn push_stack_until_text_or_choice(&mut self, id: BlockId) -> Result<Vec<Block>, RuntimeError> {
    if !self.meets_requirements_and_chance(id)? {
      return self.push_next(id);
    }
    let mut blocks = Vec::default();
    blocks.push(self.push_stack(id)?);
    let block = self.get_cuentitos_block(id)?.clone();

    match block {
      cuentitos_common::Block::Section { id: _, settings: _ } => {
        blocks.append(&mut self.update_stack()?);
        Ok(blocks)
      }
      cuentitos_common::Block::Text { id: _, settings: _ } => {
        self.update_choices()?;
        Ok(blocks)
      }
      cuentitos_common::Block::Choice { id: _, settings: _ } => {
        self.update_choices()?;
        Ok(blocks)
      }
      cuentitos_common::Block::Bucket {
        name: _,
        settings: _,
      } => {
        if let Some(next) = self.get_random_block_from_bucket(block.get_settings())? {
          blocks.append(&mut self.push_stack_until_text_or_choice(next)?);
          Ok(blocks)
        } else {
          blocks.append(&mut self.update_stack()?);
          Ok(blocks)
        }
      }
      cuentitos_common::Block::Divert { next, settings: _ } => {
        match next {
          NextBlock::BlockId(id) => {
            self.game_state.block_stack.clear();
            blocks.append(&mut self.push_stack_until_text_or_choice(id)?)
          }
          NextBlock::EndOfFile => {
            return Err(RuntimeError::StoryFinished);
          }
          NextBlock::Section(section) => {
            blocks.append(&mut self.divert(&section)?);
            blocks.append(&mut self.update_stack()?)
          }
        }
        Ok(blocks)
      }
      cuentitos_common::Block::BoomerangDivert { next, settings: _ } => {
        match next {
          NextBlock::BlockId(id) => blocks.append(&mut self.push_stack_until_text_or_choice(id)?),
          NextBlock::EndOfFile => {
            return Err(RuntimeError::StoryFinished);
          }
          NextBlock::Section(section) => {
            blocks.append(&mut self.boomerang_divert(&section)?);
            blocks.append(&mut self.update_stack()?)
          }
        }
        Ok(blocks)
      }
    }
  }

  fn get_chance(&self, id: BlockId, parent_id: Option<BlockId>) -> Result<Chance, RuntimeError> {
    let block = self.get_cuentitos_block(id)?;
    match block.get_settings().chance {
      cuentitos_common::Chance::None => Ok(Chance::None),
      cuentitos_common::Chance::Frequency(_) => {
        let parent = match parent_id {
          Some(parent_id) => self.get_cuentitos_block(parent_id)?,
          None => return Err(RuntimeError::FrequencyOutOfBucket),
        };
        let total_frequency = self.get_total_frequency(parent.get_settings())?;
        let value = self.get_frequency_with_modifier(block.get_settings())?;
        Ok(Chance::Frequency {
          value,
          total_frequency,
        })
      }
      cuentitos_common::Chance::Probability(value) => Ok(Chance::Probability(value * 100.)),
    }
  }

  fn push_stack(&mut self, id: BlockId) -> Result<Block, RuntimeError> {
    let parent_id = self
      .game_state
      .block_stack
      .last()
      .map(|stack_data| stack_data.id);
    let chance = self.get_chance(id, parent_id)?;
    let block_stack_data = BlockStackData { id, chance };

    let cuentitos_block = self.get_cuentitos_block(id)?.clone();
    if cuentitos_block.get_settings().unique {
      self.game_state.uniques_played.push(id);
    }

    self.game_state.section = cuentitos_block.get_settings().section.clone();
    let modifiers = self
      .get_cuentitos_block(block_stack_data.id)?
      .get_settings()
      .modifiers
      .clone();
    self.apply_modifiers(&modifiers)?;

    let block = self.get_block(&block_stack_data)?;
    self.game_state.block_stack.push(block_stack_data);

    Ok(block)
  }

  fn update_stack(&mut self) -> Result<Vec<Block>, RuntimeError> {
    if self.database.blocks.is_empty() {
      return Err(RuntimeError::EmptyDatabase);
    }

    if self.game_state.block_stack.is_empty() {
      return self.push_stack_until_text_or_choice(0);
    }

    let last_block_id = match self.game_state.block_stack.last() {
      Some(block_stack_data) => block_stack_data.id,
      None => return Err(RuntimeError::EmptyStack),
    };

    if last_block_id >= self.database.blocks.len() {
      return Err(RuntimeError::InvalidBlockId(last_block_id));
    }

    let settings = self
      .get_cuentitos_block(last_block_id)?
      .get_settings()
      .clone();

    if !settings.children.is_empty() {
      if let Some(child) = self.get_next_child_in_stack(&settings, 0)? {
        return self.push_stack_until_text_or_choice(child);
      }
    }

    self.pop_stack_and_push_next()
  }

  fn get_next_child_in_stack(
    &mut self,
    settings: &cuentitos_common::BlockSettings,
    next_child: usize,
  ) -> Result<Option<BlockId>, RuntimeError> {
    if next_child >= settings.children.len() {
      return Ok(None);
    }

    let id = settings.children[next_child];
    match self.get_cuentitos_block(id)? {
      cuentitos_common::Block::Choice { id: _, settings: _ } => {
        if self.game_state.choices.contains(&id) {
          Err(RuntimeError::WaitingForChoice(
            self.get_current_choices_strings()?,
          ))
        } else {
          self.get_next_child_in_stack(settings, next_child + 1)
        }
      }
      _ => Ok(Some(settings.children[0])),
    }
  }

  fn push_next(&mut self, previous_id: BlockId) -> Result<Vec<Block>, RuntimeError> {
    if self.game_state.block_stack.is_empty() {
      return self.push_stack_until_text_or_choice(previous_id + 1);
    }

    let new_block_id: usize = match self.game_state.block_stack.last() {
      Some(block_stack_data) => block_stack_data.id,
      None => return Err(RuntimeError::EmptyStack),
    };

    let parent = self.database.blocks[new_block_id].clone();

    let parent_settings = parent.get_settings();
    let mut previous_block_found = false;

    let skip_siblings = matches!(
      parent,
      cuentitos_common::Block::Bucket {
        name: _,
        settings: _,
      }
    );

    if !skip_siblings {
      for sibling in &parent_settings.children {
        if previous_block_found {
          if let cuentitos_common::Block::Choice { id: _, settings: _ } =
            self.get_cuentitos_block(*sibling)?
          {
            continue;
          }
          return self.push_stack_until_text_or_choice(*sibling);
        }
        if *sibling == previous_id {
          previous_block_found = true;
        }
      }
    }

    self.pop_stack_and_push_next()
  }

  fn pop_stack_and_push_next(&mut self) -> Result<Vec<Block>, RuntimeError> {
    let last_block_id: usize = match self.game_state.block_stack.last() {
      Some(block_stack_data) => block_stack_data.id,
      None => return Err(RuntimeError::EmptyStack),
    };

    self.game_state.block_stack.pop();
    self.push_next(last_block_id)
  }

  fn get_total_frequency(
    &self,
    bucket_settings: &cuentitos_common::BlockSettings,
  ) -> Result<u32, RuntimeError> {
    let mut total_frequency = 0;
    for child in &bucket_settings.children {
      if self.meets_requirements(*child)? {
        let child_settings = self.get_cuentitos_block(*child)?.get_settings();
        let frequency = self.get_frequency_with_modifier(child_settings)?;
        total_frequency += frequency;
      }
    }
    Ok(total_frequency)
  }

  fn get_random_block_from_bucket(
    &mut self,
    settings: &cuentitos_common::BlockSettings,
  ) -> Result<Option<BlockId>, RuntimeError> {
    let total_frequency = self.get_total_frequency(settings)?;

    if total_frequency == 0 {
      return Ok(None);
    }

    let mut random_number = self.random_with_max(total_frequency);

    for child in &settings.children {
      if self.meets_requirements(*child)? {
        let child_settings = self.get_cuentitos_block(*child)?.get_settings();
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
    self.game_state.choices = Vec::default();

    if self.game_state.block_stack.is_empty() {
      return Err(RuntimeError::EmptyStack);
    }

    let last_block_id: usize = match self.game_state.block_stack.last() {
      Some(block_stack_data) => block_stack_data.id,
      None => return Err(RuntimeError::EmptyStack),
    };

    let last_block = self.get_cuentitos_block(last_block_id)?.clone();

    let settings = last_block.get_settings();

    for child in &settings.children {
      if *child < self.database.blocks.len() {
        match self.get_cuentitos_block(*child)? {
          cuentitos_common::Block::Choice { id: _, settings: _ } => {
            if self.meets_requirements_and_chance(*child)? {
              self.game_state.choices.push(*child)
            }
          }
          cuentitos_common::Block::Bucket { name: _, settings } => {
            if let Some(picked_block) = self.get_random_block_from_bucket(&settings.clone())? {
              if let cuentitos_common::Block::Choice { id: _, settings: _ } =
                self.get_cuentitos_block(picked_block)?
              {
                self.game_state.choices.push(picked_block);
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

  use crate::{
    runtime::{self, BlockStackData, Chance},
    GameState, Output, Runtime, RuntimeError,
  };
  use cuentitos_common::{
    condition::ComparisonOperator, modifier::ModifierOperator, Block, BlockSettings, Condition,
    Config, Database, FrequencyModifier, Function, I18n, LanguageDb, LanguageId, Modifier,
    NextBlock, Requirement, Section, VariableKind,
  };
  use rand::SeedableRng;
  use rand_pcg::Pcg32;

  #[test]
  fn skip() {
    let text_1 = Block::Text {
      id: "a".to_string(),
      settings: BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: "b".to_string(),
      settings: BlockSettings {
        children: vec![2],
        ..Default::default()
      },
    };
    let choice = Block::Choice {
      id: "c".to_string(),
      settings: BlockSettings::default(),
    };

    let mut en: LanguageDb = HashMap::default();
    en.insert("a".to_string(), "Text 1".to_string());
    en.insert("b".to_string(), "Text 2".to_string());
    en.insert("c".to_string(), "Choice".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![text_1, text_2, choice],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output_text_1 = runtime::Block::Text {
      text: "Text 1".to_string(),
      settings: runtime::BlockSettings {
        id: 0,
        ..Default::default()
      },
    };
    let output_text_2 = runtime::Block::Text {
      text: "Text 2".to_string(),
      settings: runtime::BlockSettings {
        id: 1,
        ..Default::default()
      },
    };

    let output = runtime.skip().unwrap();
    let expected_output = Output {
      text_ids: vec!["a".to_string(), "b".to_string()],
      text: "Text 1\nText 2".to_string(),
      choices: vec!["Choice".to_string()],
      choices_ids: vec!["c".to_string()],
      blocks: vec![output_text_1, output_text_2],
    };

    assert_eq!(output, expected_output);
  }

  #[test]
  fn skip_works_on_end() {
    let text_1 = Block::Text {
      id: "a".to_string(),
      settings: BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: "b".to_string(),
      settings: BlockSettings::default(),
    };
    let end = Block::Divert {
      next: NextBlock::EndOfFile,
      settings: BlockSettings::default(),
    };

    let mut en: LanguageDb = HashMap::default();
    en.insert("a".to_string(), "Text 1".to_string());
    en.insert("b".to_string(), "Text 2".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![text_1, text_2, end],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output_text_1 = runtime::Block::Text {
      text: "Text 1".to_string(),
      settings: runtime::BlockSettings {
        id: 0,
        ..Default::default()
      },
    };
    let output_text_2 = runtime::Block::Text {
      text: "Text 2".to_string(),
      settings: runtime::BlockSettings {
        id: 1,
        ..Default::default()
      },
    };

    let output = runtime.skip().unwrap();
    let expected_output = Output {
      text_ids: vec!["a".to_string(), "b".to_string()],
      text: "Text 1\nText 2".to_string(),
      blocks: vec![output_text_1, output_text_2],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
  }

  #[test]
  fn skip_all() {
    let text_1 = Block::Text {
      id: "a".to_string(),
      settings: BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: "b".to_string(),
      settings: BlockSettings {
        children: vec![2],
        ..Default::default()
      },
    };
    let choice = Block::Choice {
      id: "c".to_string(),
      settings: BlockSettings::default(),
    };

    let mut en: LanguageDb = HashMap::default();
    en.insert("a".to_string(), "Text 1".to_string());
    en.insert("b".to_string(), "Text 2".to_string());
    en.insert("c".to_string(), "Choice".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![text_1, text_2, choice],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output_text_1 = runtime::Block::Text {
      text: "Text 1".to_string(),
      settings: runtime::BlockSettings {
        id: 0,
        ..Default::default()
      },
    };
    let output_text_2 = runtime::Block::Text {
      text: "Text 2".to_string(),
      settings: runtime::BlockSettings {
        id: 1,
        ..Default::default()
      },
    };

    let output = runtime.skip_all().unwrap();
    let expected_output = Output {
      text_ids: vec!["b".to_string()],
      text: "Text 2".to_string(),
      choices: vec!["Choice".to_string()],
      choices_ids: vec!["c".to_string()],
      blocks: vec![output_text_1, output_text_2],
    };

    assert_eq!(output, expected_output);
  }

  #[test]
  fn skip_all_works_on_end() {
    let text_1 = Block::Text {
      id: "a".to_string(),
      settings: BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: "b".to_string(),
      settings: BlockSettings::default(),
    };
    let end = Block::Divert {
      next: NextBlock::EndOfFile,
      settings: BlockSettings::default(),
    };

    let mut en: LanguageDb = HashMap::default();
    en.insert("a".to_string(), "Text 1".to_string());
    en.insert("b".to_string(), "Text 2".to_string());
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![text_1, text_2, end],
      i18n,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output_text_1 = runtime::Block::Text {
      text: "Text 1".to_string(),
      settings: runtime::BlockSettings {
        id: 0,
        ..Default::default()
      },
    };
    let output_text_2 = runtime::Block::Text {
      text: "Text 2".to_string(),
      settings: runtime::BlockSettings {
        id: 1,
        ..Default::default()
      },
    };

    let output = runtime.skip_all().unwrap();
    let expected_output = Output {
      text_ids: vec!["b".to_string()],
      text: "Text 2".to_string(),
      blocks: vec![output_text_1, output_text_2],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
  }

  #[test]
  fn new_runtime_works_correctly() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database);
  }

  #[test]
  fn divert_to_played_unique_goes_to_next() {
    let section_block = Block::Section {
      id: "section".to_string(),
      settings: cuentitos_common::BlockSettings {
        unique: true,
        ..Default::default()
      },
    };
    let next_block = Block::default();

    let mut sections: HashMap<Section, usize> = HashMap::default();

    let section = Section {
      name: "section".to_string(),
      parent: None,
    };

    sections.insert(section.clone(), 0);

    let database = Database {
      blocks: vec![section_block, next_block],
      sections,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };
    runtime.divert(&section).unwrap();
    assert_eq!(
      runtime.game_state.block_stack,
      vec![BlockStackData {
        id: 0,
        chance: Chance::None
      }]
    );

    runtime.divert(&section).unwrap();
    assert_eq!(
      runtime.game_state.block_stack,
      vec![BlockStackData {
        id: 1,
        chance: Chance::None
      }]
    );
  }

  #[test]
  fn divert_works_correctly() {
    let block_section_1 = Block::Section {
      id: "section_1".to_string(),
      settings: cuentitos_common::BlockSettings {
        children: vec![3],
        ..Default::default()
      },
    };
    let block_section_2 = Block::Section {
      id: "section_2".to_string(),
      settings: cuentitos_common::BlockSettings {
        children: vec![2],
        ..Default::default()
      },
    };
    let block_subsection = Block::Section {
      id: "subsection".to_string(),
      settings: cuentitos_common::BlockSettings {
        children: vec![4],
        ..Default::default()
      },
    };
    let text_1 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let mut sections: HashMap<Section, usize> = HashMap::default();

    let section_1 = Section {
      name: "section_1".to_string(),
      parent: None,
    };

    sections.insert(section_1.clone(), 0);

    let section_2 = Section {
      name: "section_2".to_string(),
      parent: None,
    };

    sections.insert(section_2.clone(), 1);

    let subsection = Section {
      name: "subsection".to_string(),
      parent: Some(Box::new(section_2)),
    };
    sections.insert(subsection.clone(), 2);
    let database = Database {
      blocks: vec![
        block_section_1,
        block_section_2,
        block_subsection,
        text_1,
        text_2,
      ],
      sections,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };
    runtime.divert(&subsection).unwrap();
    assert_eq!(
      runtime.game_state.block_stack,
      vec![
        BlockStackData {
          id: 1,
          chance: Chance::None
        },
        BlockStackData {
          id: 2,
          chance: Chance::None
        },
      ]
    );

    runtime.divert(&section_1).unwrap();
    assert_eq!(
      runtime.game_state.block_stack,
      vec![BlockStackData {
        id: 0,
        chance: Chance::None
      }]
    );
  }

  #[test]
  fn boomerang_divert_works_correctly() {
    let block_section_1 = Block::Section {
      id: "section_1".to_string(),
      settings: cuentitos_common::BlockSettings {
        children: vec![3],
        ..Default::default()
      },
    };
    let block_section_2 = Block::Section {
      id: "section_2".to_string(),
      settings: cuentitos_common::BlockSettings {
        children: vec![2],
        ..Default::default()
      },
    };
    let block_subsection = Block::Section {
      id: "subsection".to_string(),
      settings: cuentitos_common::BlockSettings {
        children: vec![4],
        ..Default::default()
      },
    };
    let text_1 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };
    let text_2 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let mut sections: HashMap<Section, usize> = HashMap::default();

    let section_1 = Section {
      name: "section_1".to_string(),
      parent: None,
    };

    sections.insert(section_1.clone(), 0);

    let section_2 = Section {
      name: "section_2".to_string(),
      parent: None,
    };

    sections.insert(section_2.clone(), 1);

    let subsection = Section {
      name: "subsection".to_string(),
      parent: Some(Box::new(section_2)),
    };
    sections.insert(subsection.clone(), 2);
    let database = Database {
      blocks: vec![
        block_section_1,
        block_section_2,
        block_subsection,
        text_1,
        text_2,
      ],
      sections,
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };
    runtime.boomerang_divert(&subsection).unwrap();
    assert_eq!(
      runtime.game_state.block_stack,
      vec![
        BlockStackData {
          id: 1,
          chance: Chance::None
        },
        BlockStackData {
          id: 2,
          chance: Chance::None
        },
      ]
    );

    runtime.boomerang_divert(&section_1).unwrap();
    assert_eq!(
      runtime.game_state.block_stack,
      vec![
        BlockStackData {
          id: 1,
          chance: Chance::None
        },
        BlockStackData {
          id: 2,
          chance: Chance::None
        },
        BlockStackData {
          id: 0,
          chance: Chance::None
        }
      ]
    );
  }

  #[test]
  fn get_choices_works_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![1, 2, 3],
      ..Default::default()
    };

    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let child_text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), choice_1, choice_2, child_text],
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      game_state: GameState {
        block_stack: vec![BlockStackData {
          id: 0,
          chance: Chance::None,
        }],
        ..Default::default()
      },
      ..Default::default()
    };

    runtime.update_choices().unwrap();
    let expected_result = vec![1, 2];
    assert_eq!(runtime.game_state.choices, expected_result);
  }
  #[test]
  fn get_choices_strings_works_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![1, 2, 3],
      ..Default::default()
    };

    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "a".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "b".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let child_text = Block::Text {
      id: "c".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
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
      game_state: GameState {
        block_stack: vec![BlockStackData {
          id: 0,
          chance: Chance::None,
        }],
        ..Default::default()
      },
      current_locale: "en".to_string(),
      ..Default::default()
    };
    runtime.update_choices().unwrap();
    let choices = runtime.get_current_choices_strings().unwrap();
    let expected_result = vec!["a".to_string(), "b".to_string()];
    assert_eq!(choices, expected_result);
  }

  #[test]
  fn updates_stack_to_first_child_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };
    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let child_1 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let child_2 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![parent.clone(), child_1.clone(), child_2.clone()],
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      game_state: GameState {
        block_stack: vec![BlockStackData {
          id: 0,
          chance: Chance::None,
        }],
        ..Default::default()
      },
      ..Default::default()
    };
    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(
      *runtime.game_state.block_stack.last().unwrap(),
      BlockStackData {
        id: 1,
        chance: Chance::None
      }
    );
  }

  #[test]
  fn update_stack_to_next_sibling_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![2, 3, 4],
      ..Default::default()
    };

    let parent = Block::Text {
      id: String::default(),
      settings,
    };

    let sibling = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let child_1 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let child_2 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let child_3 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
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
      game_state: GameState {
        block_stack: vec![
          BlockStackData {
            id: 0,
            chance: Chance::None,
          },
          BlockStackData {
            id: 2,
            chance: Chance::None,
          },
        ],
        ..Default::default()
      },
      ..Default::default()
    };

    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(
      *runtime.game_state.block_stack.last().unwrap(),
      BlockStackData {
        id: 3,
        chance: Chance::None
      }
    );
    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(
      *runtime.game_state.block_stack.last().unwrap(),
      BlockStackData {
        id: 4,
        chance: Chance::None
      }
    );
    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(
      *runtime.game_state.block_stack.last().unwrap(),
      BlockStackData {
        id: 1,
        chance: Chance::None
      }
    );
  }

  #[test]
  fn current_block_works_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };
    let parent = Block::Text {
      id: "parent".to_string(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "1".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "2".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
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
      game_state: GameState {
        block_stack: vec![BlockStackData {
          id: 0,
          chance: Chance::None,
        }],
        ..Default::default()
      },
      current_locale: "en".to_string(),
      ..Default::default()
    };

    runtime.update_choices().unwrap();
    let output = runtime.current().unwrap();
    let block = runtime
      .get_block(&BlockStackData {
        id: 0,
        chance: Chance::None,
      })
      .unwrap();
    let expected_output = crate::Output {
      text_ids: vec!["parent".to_string()],
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
      choices_ids: vec!["1".to_string(), "2".to_string()],
      blocks: vec![block],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
  }

  #[test]
  fn next_works_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };

    let parent = Block::Text {
      id: "parent".to_string(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "1".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "2".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
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
    let block = runtime
      .get_block(&BlockStackData {
        id: 0,
        chance: Chance::None,
      })
      .unwrap();

    let expected_output = crate::Output {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
      text_ids: vec!["parent".to_string()],
      choices_ids: vec!["1".to_string(), "2".to_string()],
      blocks: vec![block],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
    assert_eq!(
      runtime.game_state.block_stack,
      vec![BlockStackData {
        id: 0,
        chance: Chance::None
      }]
    );
  }

  #[test]
  fn get_random_block_from_bucket_works_correctly() {
    let settings = cuentitos_common::BlockSettings {
      children: vec![1, 2],
      ..Default::default()
    };

    let bucket = Block::Bucket {
      name: None,
      settings,
    };

    let settings = cuentitos_common::BlockSettings {
      chance: cuentitos_common::Chance::Frequency(50),
      ..Default::default()
    };

    let text_1 = Block::Text {
      id: String::default(),
      settings,
    };

    let settings = cuentitos_common::BlockSettings {
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
      game_state: GameState {
        block_stack: vec![BlockStackData {
          id: 0,
          chance: Chance::None,
        }],
        ..Default::default()
      },
      ..Default::default()
    };

    runtime.set_seed(2);

    let bucket_settings = runtime
      .get_cuentitos_block(0)
      .unwrap()
      .get_settings()
      .clone();
    let id = runtime
      .get_random_block_from_bucket(&bucket_settings)
      .unwrap()
      .unwrap();
    assert_eq!(id, 1);
    Runtime::push_stack_until_text_or_choice(&mut runtime, 1).unwrap();
    let bucket_settings = runtime
      .get_cuentitos_block(0)
      .unwrap()
      .get_settings()
      .clone();
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

    let modifiers = vec![modifier_1, modifier_2, modifier_3, modifier_4];
    let settings = cuentitos_common::BlockSettings {
      modifiers: modifiers.clone(),
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
    let current_health: i32 = runtime.get_variable("health").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_health, expected_value);
    runtime.apply_modifiers(&modifiers).unwrap();
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

    let modifiers = vec![modifier_1, modifier_2, modifier_3, modifier_4];
    let settings = cuentitos_common::BlockSettings {
      modifiers: modifiers.clone(),
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
    let current_speed: f32 = runtime.get_variable("speed").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_speed, expected_value);
    runtime.apply_modifiers(&modifiers).unwrap();
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
    let settings = cuentitos_common::BlockSettings {
      modifiers: vec![modifier.clone()],
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
    let current_bike: bool = runtime.get_variable("bike").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_bike, expected_value);
    runtime.apply_modifiers(&vec![modifier]).unwrap();
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
    let settings = cuentitos_common::BlockSettings {
      modifiers: vec![modifier.clone()],
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
    let current_message: String = runtime.get_variable("message").unwrap();
    let expected_value = variable_kind.get_default_value();
    assert_eq!(current_message, expected_value);

    runtime.apply_modifiers(&vec![modifier]).unwrap();
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
    let settings = cuentitos_common::BlockSettings {
      modifiers: vec![modifier.clone()],
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];

    let current_time_of_day: TimeOfDay = runtime.get_variable("time_of_day").unwrap();
    let expected_value = variable_kind.get_default_value().parse().unwrap();
    assert_eq!(current_time_of_day, expected_value);

    runtime.apply_modifiers(&vec![modifier]).unwrap();
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
    let settings = cuentitos_common::BlockSettings {
      frequency_modifiers: vec![freq_mod],
      chance: cuentitos_common::Chance::Frequency(100),
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
    runtime.game_state.block_stack = vec![BlockStackData {
      id: 0,
      chance: Chance::None,
    }];
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
      settings: cuentitos_common::BlockSettings {
        children: vec![1, 2, 3],
        ..Default::default()
      },
    };

    let mut sections: HashMap<Section, usize> = HashMap::default();
    sections.insert(
      Section {
        name: "section".to_string(),
        parent: None,
      },
      0,
    );

    let text_1 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        unique: true,
        ..Default::default()
      },
    };

    let text_2 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let divert = Block::Divert {
      next: NextBlock::Section(Section {
        name: "section".to_string(),
        parent: None,
      }),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![section, text_1, text_2, divert],
      sections,
      ..Default::default()
    };

    let mut runtime = Runtime::new(database);
    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(1, runtime.game_state.block_stack.last().unwrap().id);
    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(2, runtime.game_state.block_stack.last().unwrap().id);
    Runtime::update_stack(&mut runtime).unwrap();
    assert_eq!(2, runtime.game_state.block_stack.last().unwrap().id);
  }

  #[test]
  fn tags_work() {
    let tags = vec!["a_tag".to_string()];
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        tags: tags.clone(),
        ..Default::default()
      },
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let output_tags = runtime
      .progress_story()
      .unwrap()
      .blocks
      .last()
      .unwrap()
      .get_settings()
      .clone()
      .tags;
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
      settings: cuentitos_common::BlockSettings {
        functions: functions.clone(),
        ..Default::default()
      },
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let output_functions = runtime
      .progress_story()
      .unwrap()
      .blocks
      .last()
      .unwrap()
      .clone()
      .get_settings()
      .clone()
      .functions;
    assert_eq!(functions, output_functions);
  }

  #[test]
  fn invalid_id_in_stack_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.game_state.block_stack.push(BlockStackData {
      id: 1,
      chance: Chance::None,
    });
    let err = Runtime::update_stack(&mut runtime).unwrap_err();
    assert_eq!(err, RuntimeError::InvalidBlockId(1));
  }

  #[test]
  fn invalid_id_in_choice_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.game_state.choices.push(1);
    let err = runtime.pick_choice(0).unwrap_err();
    assert_eq!(err, RuntimeError::InvalidBlockId(1));
  }

  #[test]
  fn next_throws_error_if_theres_a_choice() {
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        children: vec![2],
        ..Default::default()
      },
    };

    let choice = Block::Choice {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let text_2 = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text, text_2, choice],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    Runtime::update_stack(&mut runtime).unwrap();
    let err = Runtime::update_stack(&mut runtime).unwrap_err();
    assert_eq!(
      err,
      RuntimeError::WaitingForChoice(vec!["MISSING LOCALE ``".to_string()])
    );
  }

  #[test]
  fn throws_error_when_story_finishes() {
    let text: Block = Block::Divert {
      next: NextBlock::EndOfFile,
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = Runtime::update_stack(&mut runtime).unwrap_err();
    assert_eq!(err, RuntimeError::StoryFinished);
  }

  #[test]
  fn divert_throws_error_if_section_doesnt_exist() {
    let section_key = Section {
      name: "section".to_string(),
      parent: None,
    };
    let divert = Block::Divert {
      next: NextBlock::Section(section_key.clone()),
      settings: cuentitos_common::BlockSettings::default(),
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
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![choice],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.game_state.block_stack.push(BlockStackData {
      id: 0,
      chance: Chance::None,
    });
    let err = runtime.current().unwrap_err();

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
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let runtime = Runtime::new(database);
    let err = runtime.current().unwrap_err();
    assert_eq!(err, RuntimeError::EmptyStack);
  }

  #[test]
  fn empty_database_throws_error() {
    let database = Database {
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    let err = runtime.progress_story().unwrap_err();
    assert_eq!(err, RuntimeError::EmptyDatabase);
    let err = Runtime::update_stack(&mut runtime).unwrap_err();
    assert_eq!(err, RuntimeError::EmptyDatabase);
    let err = runtime.pick_choice(0).unwrap_err();
    assert_eq!(err, RuntimeError::EmptyDatabase);
  }

  #[test]
  fn picking_choice_when_no_options_throws_error() {
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings::default(),
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
      settings: cuentitos_common::BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text],
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);
    runtime.game_state.choices.push(0);
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
      settings: cuentitos_common::BlockSettings::default(),
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
      settings: cuentitos_common::BlockSettings::default(),
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
      settings: cuentitos_common::BlockSettings::default(),
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
      settings: cuentitos_common::BlockSettings::default(),
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
      settings: cuentitos_common::BlockSettings::default(),
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

  #[test]
  fn changed_variables_works_on_progress_story() {
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        modifiers: vec![Modifier {
          variable: "health".to_string(),
          value: "10".to_string(),
          operator: ModifierOperator::Set,
        }],
        ..Default::default()
      },
    };

    let mut config = Config::default();
    config
      .variables
      .insert("health".to_string(), VariableKind::Integer);

    let database = Database {
      blocks: vec![text],
      config,
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);

    let modified_variables = runtime.progress_story().unwrap().blocks[0]
      .clone()
      .get_settings()
      .changed_variables
      .clone();

    assert_eq!(modified_variables, vec!["health".to_string()]);
  }

  #[test]
  fn changed_variables_works_on_pick_choice() {
    let text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        children: vec![1],
        ..Default::default()
      },
    };

    let choice = Block::Choice {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        modifiers: vec![Modifier {
          variable: "health".to_string(),
          value: "10".to_string(),
          operator: ModifierOperator::Set,
        }],
        children: vec![2],
        ..Default::default()
      },
    };

    let end_text = Block::Text {
      id: String::default(),
      settings: cuentitos_common::BlockSettings {
        children: vec![1],
        ..Default::default()
      },
    };

    let mut config = Config::default();
    config
      .variables
      .insert("health".to_string(), VariableKind::Integer);

    let database = Database {
      blocks: vec![text, choice, end_text],
      config,
      ..Default::default()
    };
    let mut runtime = Runtime::new(database);

    runtime.progress_story().unwrap();
    let modified_variables = runtime.pick_choice(0).unwrap().blocks[0]
      .clone()
      .get_settings()
      .changed_variables
      .clone();

    assert_eq!(modified_variables, vec!["health".to_string()]);
  }

  #[test]
  fn reset_state() {
    let mut runtime = Runtime::default();
    runtime.game_state.uniques_played.push(0);
    runtime
      .game_state
      .variables
      .insert("variable".to_string(), "true".to_string());
    assert_ne!(runtime.game_state, GameState::default());
    runtime.reset_state();
    assert_eq!(runtime.game_state, GameState::default());
  }

  #[test]
  fn reset_story() {
    let mut runtime = Runtime::default();
    runtime.random_float();
    runtime.game_state.block_stack.push(BlockStackData {
      id: 0,
      chance: Chance::None,
    });
    runtime.game_state.choices.push(1);
    runtime.game_state.section = Some(Section::default());

    assert_ne!(runtime.rng, Some(Pcg32::seed_from_u64(runtime.seed)));
    assert!(!runtime.game_state.block_stack.is_empty());
    assert!(!runtime.game_state.choices.is_empty());
    assert!(runtime.game_state.section.is_some());
    runtime.reset_story();

    assert_eq!(runtime.rng, Some(Pcg32::seed_from_u64(runtime.seed)));
    assert!(runtime.game_state.block_stack.is_empty());
    assert!(runtime.game_state.choices.is_empty());
    assert!(runtime.game_state.section.is_none());
  }

  #[test]
  fn reset_all() {
    let mut runtime = Runtime::default();
    runtime.game_state.uniques_played.push(0);
    runtime
      .game_state
      .variables
      .insert("variable".to_string(), "true".to_string());
    runtime.random_float();
    runtime.game_state.block_stack.push(BlockStackData {
      id: 0,
      chance: Chance::None,
    });
    runtime.game_state.choices.push(1);
    runtime.game_state.section = Some(Section::default());

    assert_ne!(runtime.game_state, GameState::default());
    assert_ne!(runtime.rng, Some(Pcg32::seed_from_u64(runtime.seed)));
    assert!(!runtime.game_state.block_stack.is_empty());
    assert!(!runtime.game_state.choices.is_empty());
    assert!(runtime.game_state.section.is_some());

    runtime.reset_all();

    assert_eq!(runtime.game_state, GameState::default());
    assert_eq!(runtime.rng, Some(Pcg32::seed_from_u64(runtime.seed)));
    assert!(runtime.game_state.block_stack.is_empty());
    assert!(runtime.game_state.choices.is_empty());
    assert!(runtime.game_state.section.is_none());
  }

  #[test]
  fn progress_story_works_with_next() {
    let text_1 = Block::Text {
      id: "text_1".to_string(),
      settings: BlockSettings::default(),
    };

    let settings = cuentitos_common::BlockSettings {
      children: vec![2, 3],
      ..Default::default()
    };
    let text_2 = Block::Text {
      id: "text_2".to_string(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "1".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "2".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let mut en: LanguageDb = HashMap::default();
    en.insert("1".to_string(), "1".to_string());
    en.insert("2".to_string(), "2".to_string());
    en.insert("text_1".to_string(), "text_1".to_string());
    en.insert("text_2".to_string(), "text_2".to_string());

    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![
        text_1.clone(),
        text_2.clone(),
        choice_1.clone(),
        choice_2.clone(),
      ],
      i18n,
      config: Config {
        story_progress_style: cuentitos_common::StoryProgressStyle::Next,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output = runtime.progress_story().unwrap();
    let block = runtime
      .get_block(&BlockStackData {
        id: 0,
        chance: Chance::None,
      })
      .unwrap();

    let expected_output = crate::Output {
      text: "text_1".to_string(),
      text_ids: vec!["text_1".to_string()],
      blocks: vec![block],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
    assert_eq!(
      runtime.game_state.block_stack,
      vec![BlockStackData {
        id: 0,
        chance: Chance::None
      }]
    );
  }

  #[test]
  fn progress_story_works_with_skip() {
    let text_1 = Block::Text {
      id: "text_1".to_string(),
      settings: BlockSettings::default(),
    };

    let settings = cuentitos_common::BlockSettings {
      children: vec![2, 3],
      ..Default::default()
    };
    let text_2 = Block::Text {
      id: "text_2".to_string(),
      settings,
    };

    let choice_1 = Block::Choice {
      id: "1".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let choice_2 = Block::Choice {
      id: "2".to_string(),
      settings: cuentitos_common::BlockSettings::default(),
    };

    let mut en: LanguageDb = HashMap::default();
    en.insert("1".to_string(), "1".to_string());
    en.insert("2".to_string(), "2".to_string());
    en.insert("text_1".to_string(), "text_1".to_string());
    en.insert("text_2".to_string(), "text_2".to_string());

    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), en);

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      blocks: vec![
        text_1.clone(),
        text_2.clone(),
        choice_1.clone(),
        choice_2.clone(),
      ],
      i18n,
      config: Config {
        story_progress_style: cuentitos_common::StoryProgressStyle::Skip,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      current_locale: "en".to_string(),
      ..Default::default()
    };

    let output = runtime.progress_story().unwrap();
    let block_1 = runtime
      .get_block(&BlockStackData {
        id: 0,
        chance: Chance::None,
      })
      .unwrap();

    let block_2 = runtime
      .get_block(&BlockStackData {
        id: 1,
        chance: Chance::None,
      })
      .unwrap();

    let expected_output = crate::Output {
      text: "text_1\ntext_2".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
      blocks: vec![block_1, block_2],
      text_ids: vec!["text_1".to_string(), "text_2".to_string()],
      choices_ids: vec!["1".to_string(), "2".to_string()],
      ..Default::default()
    };

    assert_eq!(output, expected_output);
    assert_eq!(
      runtime.game_state.block_stack,
      vec![BlockStackData {
        id: 1,
        chance: Chance::None
      }]
    );
  }

  #[test]
  fn rewind_story() {
    let text_1 = Block::Text {
      id: "text_1".to_string(),
      settings: BlockSettings::default(),
    };

    let text_2 = Block::Text {
      id: "text_2".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text_1.clone(), text_2.clone()],
      config: Config {
        keep_history: false,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };

    runtime.progress_story().unwrap();
    let err = runtime.rewind().unwrap_err();
    assert_eq!(err, RuntimeError::RewindWithNoHistory());

    runtime.database.config.keep_history = true;

    let initial_runtime = runtime.clone();
    runtime.progress_story().unwrap();

    assert_ne!(runtime, initial_runtime);

    runtime.rewind().unwrap();

    assert_eq!(runtime, initial_runtime);
  }

  #[test]
  fn rewind_to_choice() {
    let text_1 = Block::Text {
      id: "text_1".to_string(),
      settings: BlockSettings {
        children: vec![1],
        ..Default::default()
      },
    };

    let choice = Block::Choice {
      id: "0".to_string(),
      settings: BlockSettings {
        children: vec![2, 3],
        ..Default::default()
      },
    };

    let text_2 = Block::Text {
      id: "text_2".to_string(),
      settings: BlockSettings::default(),
    };

    let text_3 = Block::Text {
      id: "text_3".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![
        text_1.clone(),
        choice.clone(),
        text_2.clone(),
        text_3.clone(),
      ],
      config: Config {
        keep_history: true,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };

    runtime.progress_story().unwrap();

    let initial_runtime = runtime.clone();

    runtime.pick_choice(0).unwrap();
    runtime.progress_story().unwrap();

    assert_ne!(runtime, initial_runtime);

    runtime.rewind_to_choice().unwrap();

    assert_eq!(runtime, initial_runtime);
  }

  #[test]
  fn rewind_to() {
    let text_1 = Block::Text {
      id: "text_1".to_string(),
      settings: BlockSettings {
        children: vec![1],
        ..Default::default()
      },
    };

    let text_2 = Block::Text {
      id: "text_2".to_string(),
      settings: BlockSettings::default(),
    };

    let text_3 = Block::Text {
      id: "text_3".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text_1.clone(), text_2.clone(), text_3.clone()],
      config: Config {
        keep_history: true,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };

    runtime.progress_story().unwrap();
    let initial_runtime = runtime.clone();
    runtime.progress_story().unwrap();
    runtime.progress_story().unwrap();

    assert_ne!(runtime, initial_runtime);

    runtime.rewind_to(1).unwrap();

    assert_eq!(runtime, initial_runtime);

    runtime.progress_story().unwrap();
    let initial_runtime = runtime.clone();
    runtime.progress_story().unwrap();
    runtime.progress_story().unwrap();
    runtime.rewind_to(2).unwrap();
    assert_eq!(runtime, initial_runtime);
  }

  #[test]
  fn missing_locale_throws_error() {
    let mut strings: HashMap<LanguageId, LanguageDb> = HashMap::default();
    strings.insert("en".to_string(), HashMap::default());

    let i18n = I18n {
      locales: vec!["en".to_string()],
      default_locale: "en".to_string(),
      strings,
    };

    let database = Database {
      i18n,
      config: Config {
        story_progress_style: cuentitos_common::StoryProgressStyle::Skip,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };

    let runtime_error = runtime.set_locale("es").unwrap_err();
    assert_eq!(runtime_error, RuntimeError::MissingLocale("es".to_string()));
  }

  #[test]
  fn rewind_to_invalid_index_throws_error() {
    let text_1 = Block::Text {
      id: "text_1".to_string(),
      settings: BlockSettings {
        children: vec![1],
        ..Default::default()
      },
    };

    let text_2 = Block::Text {
      id: "text_2".to_string(),
      settings: BlockSettings::default(),
    };

    let text_3 = Block::Text {
      id: "text_3".to_string(),
      settings: BlockSettings::default(),
    };

    let database = Database {
      blocks: vec![text_1.clone(), text_2.clone(), text_3.clone()],
      config: Config {
        keep_history: true,
        ..Default::default()
      },
      ..Default::default()
    };

    let mut runtime = Runtime {
      database,
      ..Default::default()
    };

    let err = runtime.rewind_to(1).unwrap_err();

    assert_eq!(
      RuntimeError::RewindWithToInvalidIndex {
        index: 1,
        current_index: 0
      },
      err
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
