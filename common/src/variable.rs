use serde::{Deserialize, Serialize};

pub type VariableId = String;

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VariableKind {
  #[default]
  Integer,
  Float,
  Bool,
  Enum {
    values: Vec<String>,
  },
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

    let variable_kind = VariableKind::Enum {
      values: vec!["a-value".to_string()],
    };
    assert!(
      variable_kind
        == VariableKind::Enum {
          values: vec!["a-value".to_string()]
        }
    );
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
