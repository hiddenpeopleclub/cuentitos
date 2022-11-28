use serde::{ Serialize, Deserialize };

mod test_utils;

mod event;
pub use event::*;

mod event_choice;
pub use event_choice::*;

mod event_requirement;
pub use event_requirement::*;

mod modifier;
pub use modifier::*;

mod event_result;
pub use event_result::*;

mod resource;
pub use resource::*;

mod config;
pub use config::Config;

mod database;
pub use database::Database;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type ItemId = String;
pub type ReputationId = String;
pub type DecisionId = String;
pub type TileId = String;
pub type AchievementId = String;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TimeOfDay {
  #[default]
  Morning,
  Noon,
  Evening,
  Night
}
