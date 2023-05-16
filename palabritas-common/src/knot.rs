use crate::{Content, Stitch};


#[derive(Default)]
pub struct Knot {
  pub identifier: String,
  pub content: Vec<Box<dyn Content>>,
  pub stitches: Vec<Stitch>,
}
