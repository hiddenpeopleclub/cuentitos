use std::path::{Path, PathBuf};
use std::io::Write;
use crate::fs::File;

use std::io::Result;

pub fn save<T, S>(path: T, content: S) -> Result<()> 
where 
  T: AsRef<Path>, 
  S: AsRef<str>
{
  let mut final_path = PathBuf::new();
  final_path.push("public");
  final_path.push(path.as_ref());
  
  let mut file = File::create(final_path)?;
  
  file.write_all(content.as_ref().as_bytes())?;
  
  Ok(())
}