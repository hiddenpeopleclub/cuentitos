use crate::AchievementId;
use crate::DecisionId;
use crate::ItemId;
use crate::ReputationId;
use crate::ResourceId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Modifier {
  #[default]
  Empty,
  Resource {
    id: ResourceId,
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
