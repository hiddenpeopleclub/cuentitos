use rand_pcg::Lcg64Xsh32;
use cuentitos_common::Event;
use cuentitos_common::Database;
use rand::seq::SliceRandom;


#[derive(Debug, Default)]
pub struct Runtime {
  database: Database
}

impl Runtime {
    pub fn new(database: Database) -> Runtime {
      Runtime { database }
    }

    pub fn random_event(&mut self, mut rng: Lcg64Xsh32) -> Event {
      self.database.events.choose(&mut rng).unwrap().clone()
    }
}

#[cfg(test)]
mod test {
  use rand_pcg::Pcg32;
  use rand::SeedableRng;
  use cuentitos_common::Event;
  use crate::runtime::Runtime;
  use cuentitos_common::Database;
  use cuentitos_common::test_utils::*;

  #[test]
  fn new_runtime_accepts_database() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database)
  }
  
  #[test]
  fn gets_random_event() {
    let db = load_mp_fixture("database").unwrap();
    let database = Database::from_u8(&db).unwrap();
    let mut runtime = Runtime::new(database.clone());
    let rng = Pcg32::seed_from_u64(42);
    let event = runtime.random_event(rng);
    assert_eq!(event, database.events[0]);
  }
}
