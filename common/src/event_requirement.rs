use crate::DecisionId;
use crate::EventId;
use crate::ItemId;
use crate::ReputationId;
use crate::Variable;
use crate::TileId;
use crate::TimeOfDay;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Condition {
  #[default]
  Equals,
  HigherThan,
  LessThan,
  Depends,
  MutEx,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EventRequirement {
  #[default]
  Empty,
  Variable {
    variable: Variable,
    condition: Condition,
    amount: String,
  },
  Item {
    id: ItemId,
    condition: Condition,
    amount: String,
  },
  Reputation {
    id: ReputationId,
    condition: Condition,
    amount: String,
  },
  TimeOfDay {
    id: TimeOfDay,
    condition: Condition,
  },
  Decision {
    id: DecisionId,
    condition: Condition,
  },
  Event {
    id: EventId,
    condition: Condition,
  },
  Tile {
    id: TileId,
    condition: Condition,
  },
}
