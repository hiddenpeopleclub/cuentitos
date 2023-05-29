use palabritas_parser::parse_file_from_path;
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


pub fn compile<T, U>(source_path: T, destination_path: U) 
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
 
  println!("abt to");
  println!("path: {:?}",source_path.as_ref());
  let db = parse_file_from_path(source_path).unwrap();
  
  println!("ok from file");
  let mut buf: Vec<u8> = Vec::new();
  let mut serializer = Serializer::new(&mut buf);

  db.serialize(&mut serializer).unwrap();
  
  let destination_path = destination_path.as_ref().to_path_buf();
  let mut file = File::create(destination_path).unwrap();

  file.write_all(&buf).unwrap();
}



#[cfg(test)]
mod test {
    use crate::compile;


  #[test]
  fn compile_works_correctly() {
    compile("../examples/story-example.cuentitos", "F:/cuentitos.db");
  }
}