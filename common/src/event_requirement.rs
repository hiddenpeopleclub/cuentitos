use crate::DecisionId;
use crate::EventId;
use crate::ItemId;
use crate::ReputationId;
use crate::TimeOfDay;
use crate::Variable;
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
    value: String,
  },
  Item {
    id: ItemId,
    condition: Condition,
    value: String,
  },
  Reputation {
    id: ReputationId,
    condition: Condition,
    value: String,
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
}
