use palabritas::Rule;
use cuentitos_common::Script;
use cuentitos_common::Config;
use cuentitos_common::Database;
use i18n::I18n;
use palabritas::parse_database_str;
use rmp_serde::Serializer;
use serde::{ Serialize };
use std::fs::File;
use std::io::Write;
use std::path::Path;
use palabritas::*;

mod i18n;

pub fn compile<T, U>(source_path: T, destination_path: U) -> Result<(), Box<dyn std::error::Error>>
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  let mut db = parse_database_from_path(&source_path)?;

  I18n::process(&mut db, source_path, &destination_path)?;

  let mut buf: Vec<u8> = Vec::new();
  let mut serializer = Serializer::new(&mut buf);

  db.serialize(&mut serializer)?;

  let destination_path = destination_path.as_ref().to_path_buf();
  let mut file = File::create(destination_path)?;

  file.write_all(&buf)?;

  Ok(())
}

pub fn compile_from_str(source_code: &str, _config_toml: &str) -> Result<Database, Box<dyn std::error::Error>> {

  // let config: Config = match toml::from_str(config_toml) {
  //   Ok(d) => d,
  //   Err(err) => {
  //     return Err(Box::new(err));
  //   }
  // };

  // println!("Compiling from config: {:?}", config);
  Ok(parse_database_str(source_code, &Config::default())?)
}


pub fn compile_database<T>(source_path: T) -> Result<Database, Box<dyn std::error::Error>>
where
  T: AsRef<Path>,
{
  let db = parse_database_from_path(&source_path)?;
  
  Ok(db)
}


#[cfg(test)]
mod test {
  use crate::compile;

  #[test]
  fn compile_works_correctly() {
    compile("../examples/story-example.cuentitos", "cuentitos.db").unwrap();
  }
}
