use crate::{Block, Result};
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Database {
  pub blocks: Vec<Block>,
}

impl Database {
  pub fn from_u8(bytes: &[u8]) -> Result<Database> {
    let mut de = Deserializer::new(bytes);
    let db: std::result::Result<Database, rmp_serde::decode::Error> =
      Deserialize::deserialize(&mut de);
    match db {
      Ok(db) => Ok(db),
      Err(error) => Err(Box::new(error)),
    }
  }
}
