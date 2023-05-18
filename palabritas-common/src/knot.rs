use crate::{Content, Stitch};

#[derive(Default)]
pub struct Knot {
  pub identifier: String,
  pub content: Vec<Content>,
  pub stitches: Vec<Stitch>,
}
