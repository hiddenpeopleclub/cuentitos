use crate::{Block, BlockId, Result, SectionKey};
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, PartialEq, Deserialize, Clone)]
pub struct Database {
  pub blocks: Vec<Block>,
  pub sections: HashMap<SectionKey, BlockId>,
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
