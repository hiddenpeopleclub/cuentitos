use crate::{FrequencyModifier, Function, I18nId, Modifier, Requirement};
use core::fmt;
use serde::{Deserialize, Serialize};

pub type BlockId = usize;
pub type BucketName = String;
pub type SectionName = String;

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct SectionKey {
  pub section: String,
  pub subsection: Option<String>,
}
impl fmt::Display for SectionKey {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut key = String::new();

    key.push_str(&self.section);

    if let Some(subsection) = &self.subsection {
      key.push('/');
      key.push_str(subsection);
    }

    write!(f, "{}", key)
  }
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
  pub tags: Vec<String>,
  pub functions: Vec<Function>,
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
  pub fn get_i18n_id(&self) -> Option<I18nId> {
    match self {
      Block::Text { id, settings: _ } => Some(id.clone()),
      Block::Choice { id, settings: _ } => Some(id.clone()),
      Block::Bucket {
        name: _,
        settings: _,
      } => None,
      Block::Section { id: _, settings: _ } => None,
      Block::Subsection { id: _, settings: _ } => None,
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
