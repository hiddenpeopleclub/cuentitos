use crate::Config;
use crate::Event;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Database {
  config: Config,
  events: Vec<Event>,
  // event_id_index: HashMap<String, usize>
}
