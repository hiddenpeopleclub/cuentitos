use crate::{Content, Knot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct File {
  pub knots: Vec<Knot>,
  pub content: Vec<Content>,
}
