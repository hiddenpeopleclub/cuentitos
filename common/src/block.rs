use crate::{FrequencyModifier, I18nId, Modifier, Requirement};
use serde::{Deserialize, Serialize};

pub type BlockId = usize;
pub type SectionId = usize;
pub type BucketName = String;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum NextBlock {
  #[default]
  None,
  BlockId(BlockId),
  EndOfFile,
  Section(SectionId),
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct BlockSettings {
  pub children: Vec<BlockId>,
  pub next: NextBlock,
  pub chance: Chance,
  pub frequency_modifiers: Vec<FrequencyModifier>,
  pub requirements: Vec<Requirement>,
  pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub enum Chance {
  #[default]
  None,
  Frequency(u32),
  Probability(f32),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Block {
  Text {
    id: I18nId,
    settings: BlockSettings,
  },
  Choice {
    id: I18nId,
    settings: BlockSettings,
  },
  Bucket {
    name: Option<BucketName>,
    settings: BlockSettings,
  },
  Section {
    id: I18nId,
    settings: BlockSettings,
    subsections: Vec<BlockId>,
  },
  Subsection {
    id: I18nId,
    settings: BlockSettings,
    subsections: Vec<BlockId>,
  },
}

impl Block {
  pub fn get_settings_mut(&mut self) -> &mut BlockSettings {
    match self {
      Block::Text { id: _, settings } => settings,
      Block::Choice { id: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section {
        id: _,
        settings,
        subsections: _,
      } => settings,
      Block::Subsection {
        id: _,
        settings,
        subsections: _,
      } => settings,
    }
  }
  pub fn get_settings(&self) -> &BlockSettings {
    match self {
      Block::Text { id: _, settings } => settings,
      Block::Choice { id: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section {
        id: _,
        settings,
        subsections: _,
      } => settings,
      Block::Subsection {
        id: _,
        settings,
        subsections: _,
      } => settings,
    }
  }
}

impl Default for Block {
  fn default() -> Self {
    Block::Text {
      id: String::default(),
      settings: BlockSettings::default(),
    }
  }
}
