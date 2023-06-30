use cuentitos_runtime::*;
use palabritas::parse_modifier_str;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Console {}
impl Console {
  fn prompt(section: Option<String>, subsection: Option<String>) -> String {
    let mut line = String::new();

    let mut prompt_str = String::from("\n");

    if let Some(section) = section {
      prompt_str.push_str(&section);
    }

    if let Some(subsection) = subsection {
      prompt_str.push('/');
      prompt_str.push_str(&subsection);
    }

    prompt_str.push_str(" > ");

    print!("{}", prompt_str);
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
    let file = cuentitos_runtime::Database::from_u8(&buffer).unwrap();

    let mut runtime = Runtime::new(file);

    loop {
      let section = runtime.game_state.current_section.clone();
      let subsection = runtime.game_state.current_subsection.clone();
      let input = Self::prompt(section, subsection);

      match input.as_str() {
        "" => {
          if let Some(output_text) = runtime.next_block() {
            print_output_text(output_text);
          } else if let Some(output_text) = runtime.current_block() {
            print_output_text(output_text);
          }
        }
        "sections" => {
          println!("Sections:");
          for section in runtime.database.sections.keys() {
            println!("{:?}", section);
          }
        }
        "q" => break,
        "variables" => print_variables(&runtime),
        str => {
          if str.starts_with("set") {
            match parse_modifier_str(str) {
              Ok(modifier) => match runtime.apply_modifier(&modifier) {
                Ok(_) => {
                  print_variable(&runtime, &modifier.variable);
                }
                Err(error) => println!("{}", error),
              },
              Err(error) => println!("{}", error),
            }
          } else if str.starts_with("->") {
            let substr: String = str.chars().skip(2).collect();
            let mut splitted = substr.split('/');
            if let Some(section) = splitted.next() {
              let jump_ok;
              if let Some(subsection) = splitted.next() {
                jump_ok =
                  runtime.jump_to_section(section.to_string(), Some(subsection.to_string()));
              } else {
                jump_ok = runtime.jump_to_section(section.to_string(), None);
              }

              if jump_ok {
                if let Some(output_text) = runtime.current_block() {
                  print_output_text(output_text);
                }
              }
            }
          } else if let Ok(choice) = str.parse::<usize>() {
            if choice == 0 {
              println!("invalid option");
              continue;
            }
            if let Some(output_text) = runtime.pick_choice(choice - 1) {
              print_output_text(output_text);
            }
          } else {
            println!("Unkown command: {}", str);
          }
        }
      }
    }
  }
}

fn print_variables(runtime: &Runtime) {
  for (variable, kind) in &runtime.database.config.variables {
    match kind {
      VariableKind::Integer => {
        let int: i32 = runtime.get_variable(variable).unwrap();
        println!("{}: {}", variable, int);
      }
      VariableKind::Float => {
        let float: f32 = runtime.get_variable(variable).unwrap();
        println!("{}: {}", variable, float);
      }
      VariableKind::Bool => {
        let bool: bool = runtime.get_variable(variable).unwrap();
        println!("{}: {}", variable, bool);
      }
      _ => {
        let string: String = runtime.get_variable(variable).unwrap();
        println!("{}: {}", variable, string);
      }
    }
  }
}

fn print_variable(runtime: &Runtime, variable: &String) {
  for (runtime_variable, kind) in &runtime.database.config.variables {
    if runtime_variable == variable {
      match kind {
        VariableKind::Integer => {
          let int: i32 = runtime.get_variable(variable).unwrap();
          println!("{}: {}", variable, int);
        }
        VariableKind::Float => {
          let float: f32 = runtime.get_variable(variable).unwrap();
          println!("{}: {}", variable, float);
        }
        VariableKind::Bool => {
          let bool: bool = runtime.get_variable(variable).unwrap();
          println!("{}: {}", variable, bool);
        }
        _ => {
          let string: String = runtime.get_variable(variable).unwrap();
          println!("{}: {}", variable, string);
        }
      }
      return;
    }
  }
  println!("Variable {} doesn't exist", variable)
}
fn print_output_text(output_text: Block) {
  println!("{}", output_text.text);
  for i in 0..output_text.choices.len() {
    println!("  ({}){}", i + 1, output_text.choices[i]);
  }
}
