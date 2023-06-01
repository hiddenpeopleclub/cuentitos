use crate::{Content, Knot, OutputText, Readable};
use crate::{ContentType, Result};
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Database {
  pub knots: Vec<Knot>,
  pub content: Vec<Content>,
  #[serde(skip)]
  pub cursor: usize,
}

impl Readable for Database {
  fn get_next_output(&mut self) -> Option<OutputText> {
    if self.cursor >= self.content.len() {
      return None;
    }
    if !self.content[self.cursor].meets_requirements() {
      self.cursor += 1;
      return self.get_next_output();
    }

    let output = self.content[self.cursor].get_next_output();
    if output.is_none() {
      self.cursor += 1;
      self.get_next_output()
    } else {
      output
    }
  }

  fn pick_choice(&mut self, choice: usize) -> Option<OutputText> {
    if !self.is_in_choice() {
      return None;
    }

    let output = self.content[self.cursor].pick_choice(choice);

    if output.is_none()
      && self.cursor + choice <= self.content.len()
      && self.content[self.cursor + choice].content_type == ContentType::Choice
    {
      self.cursor += choice;
      return self.get_next_output();
    }

    output
  }

  fn is_in_choice(&self) -> bool {
    if self.content[self.cursor].content_type == ContentType::Choice {
      return true;
    }
    self.content[self.cursor].is_in_choice()
  }
}
impl Database {
  pub fn from_u8(bytes: &[u8]) -> Result<Database> {
    let mut de = Deserializer::new(bytes);
    let db: std::result::Result<Database, rmp_serde::decode::Error> =
      Deserialize::deserialize(&mut de);
    match db {
      Ok(database) => Ok(database),
      Err(error) => Err(Box::new(error)),
    }
  }
}
