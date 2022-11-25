use std::path::Path;
use cuentitos_common::Result;

pub fn compile<T,U>(source_path: T, destination_path: U)  -> Result<()>
where T: AsRef<Path>, U: AsRef<Path>
{
  check_required_files(source_path);
  Ok(())
}


pub fn check_required_files<T>(source_path: T)  -> Result<()>
where T: AsRef<Path>
{
  Ok(())
}
