use crate::OutputText;

pub trait Readable {
  fn get_next_output(&mut self) -> Option<OutputText>;
  fn pick_choice(&mut self, choice: usize) -> Option<OutputText>;
  fn is_in_choice(&self) -> bool;
}
