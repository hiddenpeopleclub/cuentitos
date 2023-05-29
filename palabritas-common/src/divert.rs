use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Divert {
  pub knot: String,
  pub stitch: Option<String>,
}
