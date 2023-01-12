use crate::AchievementId;
use crate::DecisionId;
use crate::ItemId;
use crate::ReputationId;
use crate::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Modifier {
  #[default]
  Empty,
  Resource {
    resource: Resource,
    amount: String,
  },
  Item {
    id: ItemId,
    amount: String,
  },
  Reputation {
    id: ReputationId,
    amount: String,
  },
  Decision(DecisionId),
  Achievement(AchievementId),
}
