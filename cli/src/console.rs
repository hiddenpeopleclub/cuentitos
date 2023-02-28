use cuentitos_runtime::*;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Console {}
impl Console {
  fn prompt(name: &str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
      .read_line(&mut line)
      .expect("Error: Could not read a line");

    return line.trim().to_string();
  }

  pub fn start(path: PathBuf) {
    let mut f = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    let database = Database::from_u8(&buffer).unwrap();
    
    let mut runtime = Runtime::new(database);

    loop {
      let input = Self::prompt("> ");
      let mut args = input.split(' ');

      match args.next() {
        Some("n") => {
          let event = runtime.next_event();
          match event {
            Some(event) => {
              println!("{}", event.title);
              println!("---");
              println!("{}", event.description);
            }
            None => {}
          }
        }
        Some("q") => break,
        Some(&_) => {}
        None => {}
      }
    }
  }
}
