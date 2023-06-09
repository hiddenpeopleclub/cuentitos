use crate::{I18nId, FrequencyModifier, Modifier, Requirement};
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
  pub frequency: Option<u64>,
  pub frequency_modifiers: Vec<FrequencyModifier>,
  pub requirements: Vec<Requirement>,
  pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Block {
  Text(I18nId,BlockSettings),
  Choice(I18nId,BlockSettings),
  Bucket(Option<BucketName>,BlockSettings),
}

impl Block {
  pub fn get_settings_mut(&mut self) -> &mut BlockSettings {
    match self {
      Block::Text(_, settings) => settings,
      Block::Choice(_,settings) => settings,
      Block::Bucket(_,settings) => settings,
    }
  }
}

impl Default for Block {
  fn default() -> Self {
    Block::Text (
      String::default(),
      BlockSettings::default()
    )
  }
}

