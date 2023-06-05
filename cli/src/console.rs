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
    let file = cuentitos_runtime::File::from_u8(&buffer).unwrap();

    let mut runtime = Runtime::new(file);

    loop {
      let input = Self::prompt("> ");
      let mut args = input.split(' ');

      match args.next() {
        Some("n") => {
          if let Some(output_text) = runtime.next_output() {
            print_output_text(output_text);
          }
        }
        Some("q") => break,
        Some(str) => {
          if let Ok(choice) = str.parse::<usize>() {
            if let Some(output_text) = runtime.pick_choice(choice) {
              print_output_text(output_text);
            }
          }
        }
        None => {}
      }
    }
  }
}

fn print_output_text(output_text: OutputText) {
  println!("{}", output_text.text);
  for choice in output_text.choices {
    println!("  *{}", choice);
  }
}
