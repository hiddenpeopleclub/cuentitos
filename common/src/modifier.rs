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
  Variable {
    id: VariableId,
    value: String,
  },
  Item {
    id: ItemId,
    value: String,
  },
  Reputation {
    id: ReputationId,
    value: String,
  },
  Decision(DecisionId),
  Achievement(AchievementId),
}
