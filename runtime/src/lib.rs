use cuentitos_common::*;

pub struct Runtime {
    pub database: Database,
    running: bool,
    program_counter: usize
}

impl Runtime {
    pub fn new(database: Database) -> Self {
      Self { database, running: false, program_counter: 0 }
    }

    pub fn run(&mut self) {
      self.running = true;
      self.program_counter = 0;
    }

    pub fn running(&self) -> bool {
      self.running
    }

    pub fn current_block(&self) -> Option<Block> {
      if self.running(){
        if self.database.blocks.len() < self.program_counter {
          None
        } else {
          Some(self.database.blocks[self.program_counter].clone())
        }
      }
      else {
        None
      }

    }
}

#[cfg(test)]
mod test {
  use cuentitos_common::test_case::TestCase;
  use super::*;

  #[test]
  fn accepts_database() {
    let database = cuentitos_common::Database::default();
    let runtime = Runtime::new(database.clone());
    assert_eq!(runtime.database, database);
  }

  #[test]
  fn run_initiates_runtime() {
    let database = cuentitos_common::Database::default();
    let mut runtime = Runtime::new(database.clone());

    assert_eq!(runtime.running(), false);

    runtime.run();

    assert_eq!(runtime.running(), true);
  }

  #[test]
  fn get_current_block() {
    let test_case = TestCase::from_string(
      include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
      "00000000002-two-lines-and-end.md"
    );

    let database = cuentitos_parser::parse(&test_case.script).unwrap();

    let mut runtime = Runtime::new(database);

    assert_eq!(runtime.current_block(), None);

    runtime.run();

    if let Some(Block::String(id)) = runtime.current_block() {
      assert_eq!(runtime.database.strings[id], "This is a single line");
    } else {
      assert!(false, "Expected 'This is a single line' to be returned in runtime.");
    }
  }

}
