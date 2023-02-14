use crate::parser::Parser;
use cuentitos_common::Config;
use cuentitos_common::Database;
use cuentitos_common::Result;
use rmp_serde::Serializer;
use serde::Serialize;
use std::error;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub enum CompileError {
  SourceNotDirectory,
  DestinationNotDirectory,
}

impl error::Error for CompileError {}

impl fmt::Display for CompileError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CompileError::SourceNotDirectory => write!(f, "Source is not a folder."),
      CompileError::DestinationNotDirectory => write!(f, "Destination is not a folder."),
    }
  }
}

mod parsable;
pub mod parser;

pub mod parsers;

mod i18n;
pub use i18n::I18n;


pub fn compile<T, U>(source_path: T, destination_path: U) -> Result<Parser>
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  check_required_files(&source_path).unwrap();

  // Load Config
  let mut config = Config::load(&source_path, &destination_path).unwrap();
  config.base_path = source_path.as_ref().to_path_buf();
  config.destination_path = destination_path.as_ref().to_path_buf();
  
  // Parse
  let mut parser = parser::Parser::new(config);
  parser.parse().unwrap();

  // TODO(fran): check validity of all events.

  let i18n = I18n::process(&mut parser)?;

  // // // Save to disk
  let mut buf: Vec<u8> = Vec::new();
  let mut serializer = Serializer::new(&mut buf);

  let mut db = Database {
    config: parser.config.clone(),
    ..Default::default()
  };

  for (id, event) in &parser.events {
    if let Ok(event) = event {
      let mut event = event.clone();
      event.id = id.clone();
      db.events.push(event)
    }
  }

  for (id, item) in &parser.items {
    if let Ok(item) = item {
      let mut item = item.clone();
      item.id = id.clone();
      db.items.push(item)
    }    
  }

  db.i18n = i18n;

  db.serialize(&mut serializer).unwrap();

  let destination_path = destination_path.as_ref().to_path_buf();
  let mut file = File::create(destination_path)?;

  file.write_all(&buf)?;

  Ok(parser)
}

pub fn check_required_files<T>(source_path: T) -> Result<()>
where
  T: AsRef<Path>,
{
  let base_path = source_path.as_ref().to_path_buf();
  std::fs::create_dir_all(&base_path).unwrap();

  let mut config = base_path.clone();
  config.push("config.toml");

  if !config.is_file() {
    panic!("Missing config.toml")
  }

  let mut events = base_path;
  events.push("events");
  if !events.exists() {
    panic!("Missing events folder");
  }

  Ok(())
}
