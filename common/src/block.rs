use crate::{FrequencyModifier, I18nId, Modifier, Requirement};
use serde::{Deserialize, Serialize};

pub type BlockId = usize;
pub type BucketName = String;

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct SectionKey {
  pub section: String,
  pub subsection: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum NextBlock {
  #[default]
  None,
  BlockId(BlockId),
  EndOfFile,
  Section(SectionKey),
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct BlockSettings {
  pub children: Vec<BlockId>,
  pub next: NextBlock,
  pub chance: Chance,
  pub frequency_modifiers: Vec<FrequencyModifier>,
  pub requirements: Vec<Requirement>,
  pub modifiers: Vec<Modifier>,
  pub unique: bool,
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
  },
  Subsection {
    id: I18nId,
    settings: BlockSettings,
  },
}

impl Block {
  pub fn get_settings_mut(&mut self) -> &mut BlockSettings {
    match self {
      Block::Text { id: _, settings } => settings,
      Block::Choice { id: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section { id: _, settings } => settings,
      Block::Subsection { id: _, settings } => settings,
    }
  }
  pub fn get_settings(&self) -> &BlockSettings {
    match self {
      Block::Text { id: _, settings } => settings,
      Block::Choice { id: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section { id: _, settings } => settings,
      Block::Subsection { id: _, settings } => settings,
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
