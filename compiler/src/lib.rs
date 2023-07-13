use i18n::I18n;
use palabritas::parse_database_from_path;
use rmp_serde::Serializer;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

#[cfg(test)]
mod test {
  use crate::compile;

  #[test]
  fn compile_works_correctly() {
    compile("../examples/story-example.cuentitos", "cuentitos.db").unwrap();
  }
}
