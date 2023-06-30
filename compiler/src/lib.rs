use i18n::I18n;
use palabritas::parse_database_from_path;
use rmp_serde::Serializer;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod i18n;

pub fn compile<T, U>(source_path: T, destination_path: U)
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  let db_result = parse_database_from_path(&source_path);

  let mut db = match db_result {
    Ok(db) => db,
    Err(e) => {
      println!("{}", e);
      return;
    }
  };

  if let Err(error) = I18n::process(&mut db, source_path, &destination_path) {
    println!("{}", error);
    return;
  }

  let mut buf: Vec<u8> = Vec::new();
  let mut serializer = Serializer::new(&mut buf);

  let serialize_result = db.serialize(&mut serializer);
  if serialize_result.is_err() {
    println!("{}", serialize_result.unwrap_err());
    return;
  }

  let destination_path = destination_path.as_ref().to_path_buf();
  let mut file = File::create(destination_path).unwrap();

  file.write_all(&buf).unwrap();
}

#[cfg(test)]
mod test {
  use crate::compile;

  #[test]
  fn compile_works_correctly() {
    compile("../examples/story-example.cuentitos", "cuentitos.db");
  }
}
