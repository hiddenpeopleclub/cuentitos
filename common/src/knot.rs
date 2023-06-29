use crate::{Block, Stitch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Knot {
  pub identifier: String,
  pub blocks: Vec<Block>,
  pub stitches: Vec<Stitch>,
}
