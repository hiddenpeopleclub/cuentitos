pub type VariableId = String;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VariableKind {
  #[default]
  Integer,
  Float,
  Bool,
  String,
  Enum(Vec<String>),
}

impl VariableKind {
  pub fn get_default_value(&self) -> String {
    match self {
      VariableKind::Integer => "0".to_string(),
      VariableKind::Float => "0.0".to_string(),
      VariableKind::Bool => "false".to_string(),
      VariableKind::String => "".to_string(),
      VariableKind::Enum(values) => {
        if values.is_empty() {
          "".to_string()
        } else {
          values[0].clone()
        }
      }
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Variable {
  pub id: VariableId,
  pub kind: VariableKind,
}

#[cfg(test)]
mod test {
  use crate::variable::*;

  #[test]
  fn variable_id_is_a_string() {
    let variable_id: VariableId = "health".to_string();
    assert!(variable_id == "health")
  }

  #[test]
  fn variable_kind_has_needed_values() {
    let variable_kind = VariableKind::Integer;
    assert!(variable_kind == VariableKind::Integer);

    let variable_kind = VariableKind::Float;
    assert!(variable_kind == VariableKind::Float);

    let variable_kind = VariableKind::Bool;
    assert!(variable_kind == VariableKind::Bool);

    let variable_kind = VariableKind::Enum(vec!["a-value".to_string()]);
    assert!(variable_kind == VariableKind::Enum(vec!["a-value".to_string()]));
  }

  #[test]
  fn variable_supports_id() {
    let variable = Variable {
      id: "my-variable".to_string(),
      ..Default::default()
    };

    assert_eq!(variable.id, "my-variable")
  }

  #[test]
  fn variable_supports_kind() {
    let variable = Variable {
      kind: VariableKind::Float,
      ..Default::default()
    };

    assert_eq!(variable.kind, VariableKind::Float)
  }
}
