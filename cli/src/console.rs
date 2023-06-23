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
    let file = cuentitos_runtime::Database::from_u8(&buffer).unwrap();

    let mut runtime = Runtime::new(file);

    loop {
      let prompt_str = match &runtime.game_state.current_section {
        Some(section) => match &runtime.game_state.current_subsection {
          Some(subsection) => format!("{}/{}>", section, subsection),
          None => format!("{}>", section),
        },
        None => ">".to_string(),
      };

      let input = Self::prompt(prompt_str.as_str());

      match input.as_str() {
        "" => {
          if let Some(output_text) = runtime.next_block() {
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
          if str.starts_with("set ") {
            let substr: String = str.chars().skip(4).collect();
            let mut splitted = substr.split(' ');
            if let Some(variable) = splitted.next() {
              if let Some(value) = splitted.next() {
                set_variable_value(variable, value, &mut runtime);
              }
            }
          } else if str.starts_with("->") {
            let substr: String = str.chars().skip(2).collect();
            let mut splitted = substr.split('.');
            if let Some(section) = splitted.next() {
              if let Some(subsection) = splitted.next() {
                runtime.jump_to_section(section.to_string(), Some(subsection.to_string()));
              } else {
                runtime.jump_to_section(section.to_string(), None);
              }

              if let Some(output_text) = runtime.next_block() {
                print_output_text(output_text);
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
          }
        }
      }
    }
  }
}

fn set_variable_value(variable: &str, value: &str, runtime: &mut Runtime) {
  for (variable_name, kind) in runtime.database.config.variables.clone() {
    if variable_name == variable {
      match kind {
        VariableKind::Integer => {
          let int: i32 = value.parse().unwrap();
          runtime.set_variable(variable, int).unwrap();
        }
        VariableKind::Float => {
          let float: f32 = value.parse().unwrap();
          runtime.set_variable(variable, float).unwrap();
        }
        VariableKind::Bool => {
          let bool: bool = value.parse().unwrap();
          runtime.set_variable(variable, bool).unwrap();
        }
        VariableKind::String => {
          runtime.set_variable(variable, value.to_string()).unwrap();
        }
        VariableKind::Enum(_) => match runtime.set_variable(variable, value.to_string()) {
          Ok(_) => {}
          Err(err) => println!("{}", err),
        },
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
fn print_output_text(output_text: Block) {
  println!("{}", output_text.text);
  for i in 0..output_text.choices.len() {
    println!("  ({}){}", i + 1, output_text.choices[i]);
  }
}
