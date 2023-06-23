use cuentitos_common::{Config, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct GameState {
  pub variables: HashMap<VariableId, String>,
  pub current_section: Option<String>,
  pub current_subsection: Option<String>,
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
