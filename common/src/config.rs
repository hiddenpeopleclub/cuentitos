use std::path::Path;

use std::collections::HashMap;
use std::str::FromStr;

use crate::Result;
use crate::VariableKind;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Config {
  #[serde(default)]
  pub variables: HashMap<String, VariableKind>,
  pub locales: Vec<String>,
  pub default_locale: String,
  #[serde(default)]
  pub other_texts: HashMap<String, String>,
  #[serde(default)]
  pub story_progress_style: StoryProgressStyle,
  #[serde(default)]
  pub keep_history: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StoryProgressStyle {
  #[default]
  Next,
  Skip,
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

impl FromStr for Config {
  type Err = toml::de::Error;
  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    let config: Config = match toml::from_str(s) {
      Ok(d) => d,
      Err(err) => {
        eprintln!("Unable to load data from \n`{}`", s);
        return Err(err);
      }
    };

    Ok(config)
  }
}
