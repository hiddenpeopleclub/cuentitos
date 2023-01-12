use serde::{Deserialize, Serialize};

pub type ResourceId = String;

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
  #[default]
  Integer,
  Float,
  Bool,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Resource {
  pub id: ResourceId,
  pub kind: ResourceKind,
}

#[cfg(test)]
mod test {
  use crate::resource::*;

  #[test]
  fn resource_id_is_a_string() {
    let resource_id: ResourceId = "health".to_string();
    assert!(resource_id == "health")
  }

  #[test]
  fn resource_kind_has_needed_values() {
    let resource_kind = ResourceKind::Integer;
    assert!(resource_kind == ResourceKind::Integer);

    let resource_kind = ResourceKind::Float;
    assert!(resource_kind == ResourceKind::Float);

    let resource_kind = ResourceKind::Bool;
    assert!(resource_kind == ResourceKind::Bool);
  }

  #[test]
  fn resource_supports_id() {
    let resource = Resource {
      id: "my-resource".to_string(),
      ..Default::default()
    };

    assert_eq!(resource.id, "my-resource")
  }

  #[test]
  fn resource_supports_kind() {
    let resource = Resource {
      kind: ResourceKind::Float,
      ..Default::default()
    };

    assert_eq!(resource.kind, ResourceKind::Float)
  }
}
