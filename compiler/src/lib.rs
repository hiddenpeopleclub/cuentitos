use cuentitos_common::Config;
use std::io::Write;
use std::fs::File;
use rmp_serde::Serializer;
use crate::parser::Parser;
use std::path::Path;
use cuentitos_common::Result;
use serde::Serialize;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum CompileError
{
    SourceNotDirectory,
    DestinationNotDirectory,
}

impl error::Error for CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self
        {
            CompileError::SourceNotDirectory => write!(f, "Source is not a folder."),
            CompileError::DestinationNotDirectory => write!(f, "Destination is not a folder."),
        }
        
    }
}  

mod parser;
pub mod parsers;

pub fn compile<T,U>(source_path: T, destination_path: U)  -> Result<Parser>
where T: AsRef<Path>, U: AsRef<Path>
{
  check_required_files(&source_path).unwrap();
  let mut config = Config::load(&source_path, &destination_path).unwrap();
  config.base_path = source_path.as_ref().to_path_buf();
  config.destination_path = destination_path.as_ref().to_path_buf();
  let mut parser = parser::Parser::new(config);
  parser.parse().unwrap();

  // TODO(fran): check validity of all events.

  let mut buf: Vec<u8> = Vec::new();
  let mut serializer = Serializer::new(&mut buf);

  parser.serialize(&mut serializer).unwrap();

  let destination_path = destination_path.as_ref().to_path_buf();

  let mut file = File::create(destination_path)?;
  
  file.write_all(&buf)?;

  Ok(parser)
}


pub fn check_required_files<T>(source_path: T)  -> Result<()>
where T: AsRef<Path>
{
  let base_path = source_path.as_ref().to_path_buf();
  let mut config = base_path.clone();
  config.push("config.toml");
  
  if !config.exists() { panic!("Missing config.toml") }

  let mut events = base_path;
  events.push("events");
  if !events.exists() { panic!("Missing events folder"); }

  Ok(())
}
