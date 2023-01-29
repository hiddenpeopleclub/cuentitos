use cuentitos_common::EventId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct RuntimeState {
  pub previous_events: Vec<EventId>,
  pub previous_event_cooldown: HashMap<EventId, i32>,
  pub disabled_events: Vec<EventId>,
  pub current_event: Option<usize>,
  pub current_choice: Option<usize>,
  pub current_result: Option<usize>
}
