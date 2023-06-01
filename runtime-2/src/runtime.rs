use palabritas_common::Database;
use palabritas_common::OutputText;
use palabritas_common::Readable;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Runtime {
  pub database: Database,
  #[serde(skip)]
  rng: Option<Pcg32>,
  seed: u64,
}

impl Runtime {
  pub fn new(database: Database) -> Runtime {
    Runtime {
      database,
      ..Default::default()
    }
  }

  pub fn next_text(&mut self) -> Option<OutputText> {
    self.database.get_next_output()
  }
  pub fn pick_choice(&mut self, choice: usize) -> Option<OutputText> {
    self.database.pick_choice(choice)
  }
}
