use cuentitos_common::Database;

#[derive(Debug, Default)]
pub struct Runtime {
  database: Database
}

impl Runtime {
    fn new(database: Database) -> Runtime {
      Runtime { database }
    }
}

#[cfg(test)]
mod test {
  use crate::runtime::Runtime;
  use cuentitos_common::Database;

  #[test]
  fn new_runtime_accepts_database() {
    let database = Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database)
  }
  
}
