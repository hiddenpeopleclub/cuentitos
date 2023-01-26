use cuentitos_common::ItemId;
use cuentitos_common::ReputationId;
use cuentitos_common::ResourceId;
use cuentitos_common::TimeOfDay;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct GameState {
  pub resources: HashMap<ResourceId, String>,
  pub items: HashMap<ItemId, String>,
  pub reputations: HashMap<ReputationId, i32>,
  pub time_of_day: TimeOfDay,
  pub decisions: Vec<String>,
  pub tile: String,
}
