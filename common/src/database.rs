use crate::Config;
use crate::Event;
use crate::Item;
use crate::Result;
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Database {
  pub config: Config,
  pub events: Vec<Event>,
  pub items: Vec<Item>,
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

#[cfg(test)]
mod test {
  use crate::test_utils::load_mp_fixture;
  use crate::Database;

  // #[test]
  // fn load_binary_db() {
  //   let db = load_mp_fixture("database").unwrap();
  //   let database = Database::from_u8(&db).unwrap();

  //   println!("{:?}", database);

  //   assert_eq!(database.events.len(), 5);
  // }
}
