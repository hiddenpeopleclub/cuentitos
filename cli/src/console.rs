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
  fn prompt(section: &Option<Section>) -> String {
    let mut line = String::new();

    let mut prompt_str = String::from("\n");

    if let Some(section) = section {
      prompt_str.push_str(&section.section_name);
      if let Some(subsection) = &section.subsection_name {
        prompt_str.push('/');
        prompt_str.push_str(subsection);
      }
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
      let input = Self::prompt(&runtime.game_state.section);

      match input.as_str() {
        "" => match runtime.next_block() {
          Ok((block, variables)) => {
            print_block(block);
            print_variables(&variables, &runtime)
          }
          Err(error) => print_runtime_error(error, &runtime),
        },
        "sections" => {
          println!("Sections:");
          for section in runtime.database.sections.keys() {
            println!("{:?}", section);
          }
        }
        "?" => {
          if !runtime.block_stack.is_empty() {
            match runtime.current_block() {
              Ok(block) => {
                println!("Current Block:");
                println!("  Text: {}", block.text);
                println!("  Script: {}", block.script);
              }
              Err(error) => print_runtime_error(error, &runtime),
            }
            match runtime.peek_next_block() {
              Ok((block, _)) => {
                println!("Next Block:");
                println!("  Text: {}", block.text);
                println!("  Script: {}", block.script);
              }
              Err(error) => print_runtime_error(error, &runtime),
            }
          }
          println!("Variables: ");
          print_all_variables(&runtime)
        }
        "q" => break,
        "variables" => print_all_variables(&runtime),
        str => {
          if str.starts_with("set") {
            match parse_modifier_str(str, &runtime.database.config) {
              Ok(modifier) => match runtime.apply_modifier(&modifier) {
                Ok(_) => {
                  print_variable(&modifier.variable, &runtime);
                }
                Err(error) => println!("{}", error),
              },
              Err(error) => println!("{}", error),
            }
          } else if str.starts_with("->") {
            let substr: String = str.chars().skip(2).collect();
            let mut splitted = substr.split('/');
            if let Some(section_str) = splitted.next() {
              let subsection = splitted.next();
              let section = match subsection {
                Some(subsection) => Section {
                  section_name: section_str.to_string(),
                  subsection_name: Some(subsection.to_string()),
                },
                None => Section {
                  section_name: section_str.to_string(),
                  subsection_name: None,
                },
              };
              match runtime.divert(&section) {
                Ok(_) => match runtime.next_block() {
                  Ok((block, choices)) => {
                    print_block(block);
                    print_choices(choices);
                  }
                  Err(error) => print_runtime_error(error, &runtime),
                },
                Err(error) => print_runtime_error(error, &runtime),
              }
            }
          } else if let Ok(choice) = str.parse::<usize>() {
            if choice == 0 {
              println!("invalid option");
              continue;
            }
            match runtime.pick_choice(choice - 1) {
              Ok((block, variables)) => {
                print_block(block);
                print_variables(&variables, &runtime)
              }
              Err(error) => print_runtime_error(error, &runtime),
            }
          } else {
            println!("Unkown command: {}", str);
          }
        }
      }
    }
  }
}

fn print_all_variables(runtime: &Runtime) {
  for (variable, kind) in &runtime.database.config.variables {
    match kind {
      VariableKind::Integer => {
        let int: i32 = runtime.get_variable(variable).unwrap();
        println!("  - {}: {}", variable, int);
      }
      VariableKind::Float => {
        let float: f32 = runtime.get_variable(variable).unwrap();
        println!("  - {}: {}", variable, float);
      }
      VariableKind::Bool => {
        let bool: bool = runtime.get_variable(variable).unwrap();
        println!("  - {}: {}", variable, bool);
      }
      _ => {
        let string: String = runtime.get_variable(variable).unwrap();
        println!("  - {}: {}", variable, string);
      }
    }
  }
}

fn print_variables(variables: &ModifiedVariables, runtime: &Runtime) {
  for variable in variables {
    print_variable(variable, runtime);
  }
}

fn print_variable(variable: &String, runtime: &Runtime) {
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

fn print_block(block: Block) {
  println!("{}", block.text);
  print_choices(block.choices);
}

fn print_runtime_error(error: RuntimeError, runtime: &Runtime) {
  match error {
    RuntimeError::WaitingForChoice(_) => {
      println!("Make a choice:\n");
      print_block(runtime.current_block().unwrap());
    }
    RuntimeError::InvalidChoice {
      total_choices,
      choice_picked,
    } => {
      println!(
        "Can't pick {}, because there's only {} options",
        choice_picked + 1,
        total_choices
      );
      println!("Make a choice:\n");
      print_block(runtime.current_block().unwrap());
    }
    _ => {
      println!("{}", error)
    }
  }
}

fn print_choices(choices: Vec<String>) {
  for (i, choice) in choices.iter().enumerate() {
    println!("  ({}){}", i + 1, choice);
  }
}
