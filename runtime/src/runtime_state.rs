use cuentitos_common::EventId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct RuntimeState {
  pub previous_events: HashMap<EventId, i32>,
  pub disabled_events: Vec<EventId>,
}
