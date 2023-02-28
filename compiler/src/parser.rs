use crate::parsable::Parsable;
use cuentitos_common::Item;
use cuentitos_common::{Config, Event};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Parser {
  pub config: Config,
  pub events: HashMap<String, Result<Event, String>>,
  pub items: HashMap<String, Result<Item, String>>,
  pub i18n: HashMap<String, HashMap<String, Option<String>>>,
}

impl Parser {
  pub fn new(config: Config) -> Parser {
    Parser {
      config,
      ..Default::default()
    }
  }

  pub fn parse(&mut self) -> Result<(), String> {
    parse_type::<crate::parsers::Event, cuentitos_common::Event>(
      "events",
      &self.config,
      &mut self.events,
    )
    .unwrap();
    parse_type::<crate::parsers::Item, cuentitos_common::Item>(
      "items",
      &self.config,
      &mut self.items,
    )
    .unwrap();
    Ok(())
  }
}

fn load_file(path: &DirEntry) -> Result<String, String> {
  match fs::read(path.path()) {
    Ok(content) => match String::from_utf8_lossy(&content).parse::<String>() {
      Ok(content) => Ok(content),
      Err(err) => Err(format!(
        "Couldn't load '{}', Error: {}",
        path.path().to_str().ok_or("")?,
        err.to_string()
      )),
    },
    Err(err) => Err(format!(
      "Couldn't load '{}', Error: {}",
      path.path().to_str().ok_or("")?,
      err.to_string()
    )),
  }
}

fn parse_type<T, S>(
  directory: &str,
  config: &Config,
  collection: &mut HashMap<String, Result<S, String>>,
) -> Result<(), String>
where
  T: Parsable<S>,
{
  let paths = paths(directory, config);

  for path in paths {
    let id = parse_id(&path);
    if let Some(id) = id {
      let content = load_file(&path)?;
      let parsed = T::parse(content, config);
      collection.insert(id.to_string(), parsed);
    }
  }
  Ok(())
}

fn parse_id(path: &DirEntry) -> Option<String> {
  Some(path.file_name().to_str()?.split('.').next()?.to_string())
}

fn paths(directory: &str, config: &Config) -> Vec<DirEntry> {
  let mut base_path = config.base_path.clone();
  base_path.push(directory);

  let mut paths: Vec<_> = fs::read_dir(&base_path)
    .unwrap()
    .map(|r| r.unwrap())
    .filter(|r| r.file_name().to_str().unwrap().chars().nth(0).unwrap() != '.')
    .collect();

  paths.sort_by_key(|dir| dir.path());
  paths
}

#[cfg(test)]
mod test {
  use crate::parser::*;

  #[test]
  fn new_receives_config() {
    let parser = Parser::new(Config::default());
    assert_eq!(parser.config, Config::default());
  }

  #[test]
  fn parse_loads_events() {
    let config = Config::load("fixtures", "fixtures-build").unwrap();
    let mut parser = Parser::new(config);
    parser.parse().unwrap();
    assert_eq!(parser.events.len(), 6);
  }

  #[test]
  fn parse_loads_items() {
    let config = Config::load("fixtures", "fixtures-build").unwrap();
    let mut parser = Parser::new(config);
    parser.parse().unwrap();
    assert_eq!(parser.items.len(), 3);
  }
}
