use crate::{FrequencyModifier, Function, I18nId, Modifier, Requirement};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub type BlockId = usize;
pub type BucketName = String;
pub type SectionName = String;

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Section {
  pub section_name: SectionName,
  pub subsection_name: Option<SectionName>,
}
impl fmt::Display for Section {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut key = String::new();

    key.push_str(&self.section_name);

    if let Some(subsection) = &self.subsection_name {
      key.push('/');
      key.push_str(subsection);
    }

    write!(f, "{}", key)
  }
}
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum NextBlock {
  #[default]
  EndOfFile,
  BlockId(BlockId),
  Section(Section),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Script {
  pub file: String,
  pub line: usize,
  pub col: usize,
}

impl Default for Script {
  fn default() -> Self {
    Self {
      file: Default::default(),
      line: 1,
      col: 1,
    }
  }
}

impl Display for Script {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}:{}", self.file, self.line, self.col)
  }
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct BlockSettings {
  pub children: Vec<BlockId>,
  pub chance: Chance,
  pub frequency_modifiers: Vec<FrequencyModifier>,
  pub requirements: Vec<Requirement>,
  pub modifiers: Vec<Modifier>,
  pub unique: bool,
  pub tags: Vec<String>,
  pub functions: Vec<Function>,
  pub script: Script,
  pub section: Option<Section>,
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
    id: SectionName,
    settings: BlockSettings,
  },
  Subsection {
    id: SectionName,
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

impl Block {
  pub fn get_settings_mut(&mut self) -> &mut BlockSettings {
    match self {
      Block::Text { id: _, settings } => settings,
      Block::Choice { id: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section { id: _, settings } => settings,
      Block::Subsection { id: _, settings } => settings,
      Block::Divert { next: _, settings } => settings,
      Block::BoomerangDivert { next: _, settings } => settings,
    }
  }
  pub fn get_settings(&self) -> &BlockSettings {
    match self {
      Block::Text { id: _, settings } => settings,
      Block::Choice { id: _, settings } => settings,
      Block::Bucket { name: _, settings } => settings,
      Block::Section { id: _, settings } => settings,
      Block::Subsection { id: _, settings } => settings,
      Block::Divert { next: _, settings } => settings,
      Block::BoomerangDivert { next: _, settings } => settings,
    }
  }
  pub fn get_i18n_id(&self) -> Option<I18nId> {
    match self {
      Block::Text { id, settings: _ } => Some(id.clone()),
      Block::Choice { id, settings: _ } => Some(id.clone()),
      _ => None,
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
