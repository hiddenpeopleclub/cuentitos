use std::path::Path;
use std::path::PathBuf;

use std::collections::HashMap;

use crate::Result;
use crate::VariableKind;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Config {
  #[serde(skip)]
  pub base_path: PathBuf,
  #[serde(skip)]
  pub destination_path: PathBuf,
  #[serde(default)]
  pub variables: HashMap<String, VariableKind>,
  #[serde(default)]
  pub reputations: Vec<String>,
  #[serde(default)]
  pub settings: Vec<String>,
  #[serde(default)]
  pub runtime: RuntimeConfig,
  pub locales: Vec<String>,
  pub default_locale: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct RuntimeConfig {
  #[serde(default)]
  pub chosen_event_frequency_penalty: i32,
  #[serde(default)]
  pub event_frequency_cooldown: u32,
  #[serde(default)]
  pub met_requirement_frequency_boost: u32,
}

impl Default for RuntimeConfig {
  fn default() -> RuntimeConfig {
    RuntimeConfig {
      chosen_event_frequency_penalty: -100,
      event_frequency_cooldown: 10,
      met_requirement_frequency_boost: 50,
    }
  }
}

impl Config {
  pub fn load<T, U>(source_path: T, destination_path: U) -> Result<Config>
  where
    T: AsRef<Path>,
    U: AsRef<Path>,
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
  use crate::Config;
  use crate::VariableKind;

  // #[test]
  // fn loads_config_from_toml() {
  //   // TODO(fran): Fix this
  //   let config = Config::load("fixtures", "fixtures-build").expect("Cannot load fixture");
  //   let mut expected = Config::default();
  //   expected.base_path.push("fixtures");
  //   expected.destination_path.push("fixtures-build");
  //   expected
  //     .variables
  //     .insert("health".to_string(), VariableKind::Integer);
  //   expected
  //     .variables
  //     .insert("happy".to_string(), VariableKind::Bool);
  //   expected
  //     .variables
  //     .insert("tiles".to_string(), VariableKind::Enum { values: vec!["forest".to_string()] });
  //   expected.reputations = vec!["rep_1".to_string(), "rep_2".to_string()];
  //   expected.settings.push("character".to_string());
  //   expected.settings.push("character-voice".to_string());
  //   expected.locales = vec!["en".to_string()];
  //   expected.default_locale = "en".to_string();

  //   assert_eq!(config, expected);
  // }
}
