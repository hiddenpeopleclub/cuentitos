use crate::Content;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Stitch {
  pub identifier: String,
  pub content: Vec<Content>,
}
