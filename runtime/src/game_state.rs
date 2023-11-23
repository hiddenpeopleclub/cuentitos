use cuentitos_common::{BlockId, Config, Section, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::BlockStackData;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct GameState {
  pub variables: HashMap<VariableId, String>,
  pub uniques_played: Vec<BlockId>,
  pub block_stack: Vec<BlockStackData>,
  pub section: Option<Section>,
  pub choices: Vec<BlockId>,
}

impl GameState {
  pub fn from_config(config: &Config) -> Self {
    let mut variables = HashMap::default();
    for (key, kind) in &config.variables {
      variables.insert(key.clone(), kind.get_default_value());
    }
    GameState {
      variables,
      ..Default::default()
    }
  }
}
