use std::io::Read;
use rmp_serde::Deserializer;
use std::path::Path;
use std::io::Write;
use std::fs::File;
use rmp_serde::Serializer;
use crate::runtime_state::RuntimeState;
use serde::{Deserialize, Serialize};
use rand_pcg::Lcg64Xsh32;
use cuentitos_common::Event;
use cuentitos_common::Database;
use rand::seq::SliceRandom;
use std::fs;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Runtime {
  database: Database,
  state: RuntimeState
}

impl Runtime {
    pub fn new(database: Database) -> Runtime {
      Runtime { database, ..Default::default() }
    }

    pub fn random_event(&mut self, mut rng: &mut Lcg64Xsh32) -> Option<Event> {
      match self.database.events.choose(rng) {
        Some(event) => {
          // Find index to add to previous events
          let index = self.database.events.iter().position(|r| r == event).unwrap();
          self.state.previous_events.push(index);
          Some(event.clone())
        },
        None => None
      }

      // Select events that meet the current state
      // Calculate probability of each event given current state
      //  - Augment frequency if conditions met
      //  - Reduce frequency if already shown, with cool down
    }

    pub fn save<T>(&self, path: T) -> cuentitos_common::Result<()> 
    where
      T: AsRef<Path>
    {
      let mut buf: Vec<u8> = Vec::new();
      let mut serializer = Serializer::new(&mut buf);
      self.serialize(&mut serializer)?;
      let mut file = File::create(path)?;
      file.write_all(&buf)?;
      Ok(())
    }

    pub fn load<T>(path: T) -> cuentitos_common::Result<Runtime>
    where
      T: AsRef<Path>
    {
      let mut f = File::open(&path)?;
      let metadata = fs::metadata(&path)?;
      let mut buffer = vec![0; metadata.len() as usize];
      f.read_exact(&mut buffer)?;
      let buf: &[u8] = &buffer;
      let mut de = Deserializer::new(buf);
      let runtime: std::result::Result<Runtime,rmp_serde::decode::Error> = Deserialize::deserialize(&mut de);
      match runtime {
        Ok(runtime) => Ok(runtime),
        Err(error) => Err(Box::new(error)),
      }
    }
}

#[cfg(test)]
mod test {
  use cuentitos_common::Event;
  use rand_pcg::Pcg32;
  use rand::SeedableRng;
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
    let mut rng = Pcg32::seed_from_u64(42);
    let event = runtime.random_event(&mut rng).unwrap();
    assert_eq!(event, database.events[0]);
  }

  #[test]
  fn runtime_can_be_saved_and_loaded() {
    let mut path = std::env::temp_dir().to_path_buf();
    path.push("state_save.mp");

    let db = load_mp_fixture("database").unwrap();
    let database = Database::from_u8(&db).unwrap();
    let runtime = Runtime::new(database.clone());
  
    runtime.save(&path).unwrap();

    let runtime2 = Runtime::load(path).unwrap();

    assert_eq!(runtime, runtime2);
  }

  #[test]
  fn choosing_event_stores_it_in_state(){
    let mut db = Database::default();
    db.events.push(Event { id: "event-1".to_string(), ..Default::default() });
    db.events.push(Event { id: "event-2".to_string(), ..Default::default() });
    let mut runtime = Runtime::new(db);
    let mut rng = Pcg32::seed_from_u64(42);
    
    runtime.random_event(&mut rng).unwrap();
    assert_eq!(runtime.state.previous_events, [0]);
    runtime.random_event(&mut rng).unwrap();
    assert_eq!(runtime.state.previous_events, [0,1]);
  }
}
