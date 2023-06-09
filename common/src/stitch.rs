use crate::Block;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Stitch {
  pub identifier: String,
  pub blocks: Vec<Block>,
}
