use crate::{Content, Stitch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Knot {
  pub identifier: String,
  pub content: Vec<Content>,
  pub stitches: Vec<Stitch>,
}
