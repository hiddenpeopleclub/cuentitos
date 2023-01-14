use rmp_serde::Deserializer;
use crate::Config;
use crate::Event;
use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Database {
  pub config: Config,
  pub events: Vec<Event>,
  // event_id_index: HashMap<String, usize>
}

impl Database {
  fn from_u8(bytes: &[u8]) -> Result<Database> {
    let mut de = Deserializer::new(bytes);
    let db: std::result::Result<Database,rmp_serde::decode::Error> = Deserialize::deserialize(&mut de);
    match db {
      Ok(database) => Ok(database),
      Err(error) => Err(Box::new(error)),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::Database;
  use crate::test_utils::load_mp_fixture;

  #[test]
  fn load_binary_db() {
    let db = load_mp_fixture("database").unwrap();
    let database = Database::from_u8(&db).unwrap();

    println!("{:?}", database);

    assert_eq!(database.events.len(), 5);
  }
}