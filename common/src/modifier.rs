use crate::AchievementId;
use crate::DecisionId;
use crate::ItemId;
use crate::ReputationId;
use crate::VariableId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Modifier {
  #[default]
  Empty,
  Resource {
    id: VariableId,
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
