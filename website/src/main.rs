use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::io::Result;

mod index;
pub mod util;
fn main() -> Result<()> {
  let path = Path::new("./public");

  match fs::remove_dir_all(&path) {
    Ok(_) => println!("- public directory and all contents removed successfully"),
    Err(e) => {
      match e.kind() {
          ErrorKind::NotFound => {},
          _ => { panic!("Error: {}", e) }
      }      
    },
  }

  match fs::create_dir_all(&path) {
    Ok(_) => println!("- new public directory created successfully"),
    Err(e) => println!(" - error creating directory: {}", e),
  }

  index::generate()?;
  // adrs::generate();
  // docs::generate();

  Ok(())
}
