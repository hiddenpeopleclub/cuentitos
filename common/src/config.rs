use std::path::Path;
use std::path::PathBuf;

use std::collections::HashMap;

use crate::ResourceKind;
use crate::Result;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Config {
  #[serde(skip)]
  pub base_path: PathBuf,
  #[serde(skip)]
  pub destination_path: PathBuf,
  pub resources: HashMap<String, ResourceKind>,
  pub reputations: Vec<String>,
  pub tiles: Vec<String>,
}

impl Config {
  pub fn load<T, U>(source_path: T, destination_path: U) -> Result<Config>
  where
    T: AsRef<Path>,
    U: AsRef<Path>,
  {
    let mut filename = source_path.as_ref().to_path_buf();
    filename.push("config.toml");
    let contents = match std::fs::read_to_string(&filename) {
      Ok(c) => c,
      Err(err) => {
        eprintln!("Could not read config file `{}`", filename.display());
        return Err(Box::new(err));
      }
    };

    let mut config: Config = match toml::from_str(&contents) {
      Ok(d) => d,
      Err(err) => {
        eprintln!("Unable to load data from `{}`", filename.display());
        return Err(Box::new(err));
      }
    };

    config.base_path = source_path.as_ref().to_path_buf();
    config.destination_path = destination_path.as_ref().to_path_buf();
    Ok(config)
  }
}

#[cfg(test)]
mod test {
  use crate::config::*;

  #[test]
  fn loads_config_from_toml() {
    let config = Config::load("fixtures", "fixtures-build").expect("Cannot load fixture");
    let mut expected = Config::default();
    expected.base_path.push("fixtures");
    expected.destination_path.push("fixtures-build");
    expected
      .resources
      .insert("health".to_string(), ResourceKind::Integer);
    expected
      .resources
      .insert("happy".to_string(), ResourceKind::Bool);
    expected.reputations = vec!["rep-1".to_string(), "rep-2".to_string()];
    expected.tiles.push("forest".to_string());
    assert_eq!(config, expected);
  }
}
