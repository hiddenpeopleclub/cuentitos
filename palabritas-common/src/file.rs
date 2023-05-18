use crate::{Content, Knot};

#[derive(Default)]
pub struct File {
  pub knots: Vec<Knot>,
  pub content: Vec<Content>,
}
