use cuentitos_common::{ Config, Event };
use std::collections::HashMap;
use std::str::FromStr;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Parser 
{
  pub config: Config,
  pub events: HashMap<String, Result<Event, String>>
}

impl Parser {
  pub fn new(config: Config) -> Parser {
    Parser {
      config,
      ..Default::default()
    }
  }

  pub fn parse(&mut self) -> Result<(), String> {
    let mut events_path = self.config.base_path.clone();
    events_path.push("events");
    let mut paths: Vec<_> = fs::read_dir(&events_path).unwrap()
      .map(|r| r.unwrap())
      .collect();
    paths.sort_by_key(|dir| dir.path());

    for path in paths {
      let id = &path.file_name();
      let id = id.to_str().unwrap()[3..].split(".").next().unwrap();
      let id = String::from_str(id).unwrap();

      if id != "" {
        let content = match fs::read(path.path()) {
          Ok(data) => data,
          Err(err) => { panic!("Error reading '{}': {}", path.path().to_str().unwrap(), err) }
        };
        let content = String::from_utf8_lossy(&content).parse::<String>().unwrap();

        let event = crate::parsers::Event::parse(content, &self.config);
        self.events.insert(id, event);        
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::parser::*;

  #[test]
  fn new_receives_config() {
    let parser = Parser::new( Config::default() );
    assert_eq!(parser.config, Config::default());
  }

  #[test]
  fn parse_loads_events() {
    let config = Config::load("fixtures", "fixtures-build").unwrap();
    let mut parser = Parser::new( config );
    parser.parse();
    assert_eq!(parser.events.len(), 5);
  }
}
