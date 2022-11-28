use std::path::Path;

use std::collections::HashMap;


use cuentitos_common::ResourceKind;
use cuentitos_common::Result;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Config {
  resources: HashMap<String, ResourceKind>
}

pub fn load<T>(source_path: T)  -> Result<Config>
where T: AsRef<Path>
{
  let mut filename = source_path.as_ref().to_path_buf();
  filename.push("config.toml");
  let contents = match std::fs::read_to_string(&filename) {
    Ok(c) => c,
    Err(err) => {
        eprintln!("Could not read config file `{}`", filename.display());
        return Err(Box::new(err))
    }
  };

  let config: Config = match toml::from_str(&contents) {
    Ok(d) => d,
    Err(err) => {
        eprintln!("Unable to load data from `{}`", filename.display());
        return Err(Box::new(err))
    }
  };

  Ok(config)
}

#[cfg(test)]
mod test {
  use crate::config::*;

#[test]
  fn loads_config_from_toml() {
    let config = load("fixtures").expect("Cannot load fixture");
    let mut expected = Config::default();
    expected.resources.insert("health".to_string(), ResourceKind::Integer);
    assert_eq!(config, expected);
  }
}
