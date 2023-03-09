use cuentitos_common::ItemId;
use cuentitos_common::ReputationId;
use cuentitos_common::TimeOfDay;
use cuentitos_common::VariableId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct GameState {
  pub variables: HashMap<VariableId, String>,
  pub items: HashMap<ItemId, u8>,
  pub reputations: HashMap<ReputationId, i32>,
  pub time_of_day: TimeOfDay,
  pub decisions: Vec<String>,
  pub tile: String,
}
