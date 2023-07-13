use crate::{Block, BlockId, Config, I18n, Result, Section};
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, PartialEq, Deserialize, Clone)]
pub struct Database {
  pub blocks: Vec<Block>,
  pub sections: HashMap<Section, BlockId>,
  pub config: Config,
  pub i18n: I18n,
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
