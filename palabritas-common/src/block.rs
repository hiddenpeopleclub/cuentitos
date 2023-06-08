use crate::{FrequencyModifier, Modifier, Requirement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct Definition {
  pub i18n_id: String,
  pub navigation: Navigation,
  pub settings: BlockSettings,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct BucketDefinition {
  pub name: Option<String>,
  pub navigation: Navigation,
  pub settings: BlockSettings,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub enum Block {
  #[default]
  None,
  Text(Definition),
  Choice(Definition),
  Bucket(BucketDefinition),
}

impl Block {
  pub fn get_navigation(&self) -> Option<&Navigation> {
    match self {
      Block::None => None,
      Block::Text(definition) => Some(&definition.navigation),
      Block::Choice(definition) => Some(&definition.navigation),
      Block::Bucket(definition) => Some(&definition.navigation),
    }
  }

  pub fn get_navigation_mut(&mut self) -> Option<&mut Navigation> {
    match self {
      Block::None => None,
      Block::Text(definition) => Some(&mut definition.navigation),
      Block::Choice(definition) => Some(&mut definition.navigation),
      Block::Bucket(definition) => Some(&mut definition.navigation),
    }
  }

  pub fn get_settings(&self) -> Option<&BlockSettings> {
    match self {
      Block::None => None,
      Block::Text(definition) => Some(&definition.settings),
      Block::Choice(definition) => Some(&definition.settings),
      Block::Bucket(definition) => Some(&definition.settings),
    }
  }

  pub fn get_settings_mut(&mut self) -> Option<&mut BlockSettings> {
    match self {
      Block::None => None,
      Block::Text(definition) => Some(&mut definition.settings),
      Block::Choice(definition) => Some(&mut definition.settings),
      Block::Bucket(definition) => Some(&mut definition.settings),
    }
  }
}
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct BlockSettings {
  pub frequency: Option<u64>,
  pub frequency_modifiers: Vec<FrequencyModifier>,
  pub requirements: Vec<Requirement>,
  pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Navigation {
  pub children: Vec<BlockId>,
  pub next: NavigationNext,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct BlockId(pub usize);

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct SectionId(pub String);

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum NavigationNext {
  #[default]
  None,
  BlockId(BlockId),
  EOF,
  Section(SectionId),
}
