use palabritas::parse_database_from_path;
use rmp_serde::Serializer;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn compile<T, U>(source_path: T, destination_path: U)
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  let db_result = parse_database_from_path(source_path);

  let db = match db_result {
    Ok(db) => db,
    Err(_) => {
      println!("{}", db_result.unwrap_err());
      return;
    }
  };

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
