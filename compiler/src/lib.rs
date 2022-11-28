use std::path::Path;
use cuentitos_common::Result;

mod config;


pub fn compile<T,U>(source_path: T, _destination_path: U)  -> Result<()>
where T: AsRef<Path>, U: AsRef<Path>
{
  check_required_files(&source_path).unwrap();
  let config = config::load(&source_path);
  println!("Config: {:?}", config);
  Ok(())
}


pub fn check_required_files<T>(source_path: T)  -> Result<()>
where T: AsRef<Path>
{
  let base_path = source_path.as_ref().to_path_buf();
  let mut config = base_path.clone();
  config.push("config.toml");
  
  if !config.exists() { panic!("Missing config.toml") }

  let mut events = base_path.clone();
  events.push("events");
  if !events.exists() { panic!("Missing events folder"); }

  Ok(())
}
