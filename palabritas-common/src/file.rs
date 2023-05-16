use crate::{Knot, Content};

#[derive(Default)]
pub struct File {
  pub knots: Vec<Knot>,
  pub content: Vec<Box<dyn Content>>,
}
