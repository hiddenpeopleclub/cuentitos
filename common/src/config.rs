use std::path::Path;

use std::collections::HashMap;

use crate::Result;
use crate::VariableKind;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Config {
  #[serde(default)]
  pub variables: HashMap<String, VariableKind>,
  pub locales: Vec<String>,
  pub default_locale: String,
}

impl Config {
  pub fn load<T>(source_path: T) -> Result<Config>
  where
    T: AsRef<Path>,
  {
    let mut filename = source_path.as_ref().to_path_buf();
    filename.push("cuentitos.toml");
    let contents = match std::fs::read_to_string(&filename) {
      Ok(c) => c,
      Err(err) => {
        eprintln!("Could not read config file `{}`", filename.display());
        return Err(Box::new(err));
      }
    };

    let config: Config = match toml::from_str(&contents) {
      Ok(d) => d,
      Err(err) => {
        eprintln!("Unable to load data from `{}`", filename.display());
        return Err(Box::new(err));
      }
    };

    Ok(config)
  }
}
