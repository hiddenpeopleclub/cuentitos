use crate::{FrequencyModifier, Modifier, Requirement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct BlockDefinition {
    i18n_id: String,
    navigation: Navigation,
    settings: BlockSettings,  
}

pub enum Block {
  #[default]
  None,
  Text(Definition),
  Choice(Definition),
  Bucket(BucketDefinition)
}

impl Block {
  pub fn get_navigation(&self) -> Option<&Navigation> {
    match self {
      Block::None => None,
      Block::Text {
        i18n_id: _,
        navigation,
        settings: _,
      } => Some(navigation),
      Block::Choice {
        i18n_id: _,
        navigation,
        settings: _,
      } => Some(navigation),
    }
  }

  pub fn get_navigation_mut(&mut self) -> Option<&mut Navigation> {
    match self {
      Block::None => None,
      Block::Text {
        i18n_id: _,
        navigation,
        settings: _,
      } => Some(navigation),
      Block::Choice {
        i18n_id: _,
        navigation,
        settings: _,
      } => Some(navigation),
    }
  }

  pub fn get_settings(&self) -> Option<&BlockSettings> {
    match self {
      Block::Text {
        i18n_id: _,
        navigation: _,
        settings,
      } => Some(settings),
      Block::Choice {
        i18n_id: _,
        navigation: _,
        settings,
      } => Some(settings),
      _ => None,
    }
  }

  pub fn get_settings_mut(&mut self) -> Option<&mut BlockSettings> {
    match self {
      Block::Text {
        i18n_id: _,
        navigation: _,
        settings,
      } => Some(settings),
      Block::Choice {
        i18n_id: _,
        navigation: _,
        settings,
      } => Some(settings),
      _ => None,
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
  pub children: Vec<usize>,
  pub index: usize,
  pub next: Option<usize>,
}
