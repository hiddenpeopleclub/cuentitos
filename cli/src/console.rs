use cuentitos_runtime::*;
use palabritas::parse_modifier_str;
use rustyline::{error::ReadlineError, history::FileHistory};
use rustyline::{DefaultEditor, Editor};
use std::fs;
use std::fs::File;
use std::io::Read;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Console {}
impl Console {
  fn prompt(
    rl: &mut Editor<(), FileHistory>,
    section: &Option<Section>,
  ) -> Result<String, ReadlineError> {
    let mut prompt_str = String::from("\n");

    if let Some(section) = section {
      prompt_str.push_str(&section.section_name);
      if let Some(subsection) = &section.subsection_name {
        prompt_str.push('/');
        prompt_str.push_str(subsection);
      }
    }

    prompt_str.push_str(" > ");

    rl.readline(&prompt_str)
  }

  pub fn start(path: PathBuf) {
    let mut f = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    let file = cuentitos_runtime::Database::from_u8(&buffer).unwrap();

    let mut runtime = Runtime::new(file);

    let mut rl = DefaultEditor::new().unwrap();
    rl.load_history("history.txt").unwrap_or_default();

    loop {
      let read_line = Self::prompt(&mut rl, &runtime.game_state.section);

      match read_line {
        Ok(line) => {
          rl.add_history_entry(line.as_str()).unwrap();
          run_command(line, &mut runtime);
        }
        Err(ReadlineError::Interrupted) => break,
        Err(ReadlineError::Eof) => break,
        Err(err) => {
          println!("{:?}", err);
          break;
        }
      }
    }

    rl.save_history("history.txt").unwrap();
  }
}

fn run_command(input: String, runtime: &mut Runtime) {
  match input.trim() {
    "" => match runtime.progress_story() {
      Ok(output) => {
        print_output(output);
      }
      Err(error) => print_runtime_error(error, runtime),
    },
    "sections" => {
      println!("Sections:");
      for section in runtime.database.sections.keys() {
        println!("{:?}", section);
      }
    }
    "?" => {
      if !runtime.block_stack.is_empty() {
        println!("Current Block:");
        print_output_result(runtime.current(), runtime);
        println!("Next Block:");
        print_output_result(runtime.peek_next(), runtime);
      }
      println!("Variables: ");
      print_all_variables(runtime)
    }
    "variables" => print_all_variables(runtime),
    str => {
      if str.starts_with("set") {
        match parse_modifier_str(str, &runtime.database.config) {
          Ok(modifier) => match runtime.apply_modifier(&modifier) {
            Ok(_) => {
              print_variable(&modifier.variable, runtime);
            }
            Err(error) => println!("{}", error),
          },
          Err(error) => println!("{}", error),
        }
      } else if str.starts_with("->") {
        let substr: String = str.chars().skip(2).collect();
        let mut splitted = substr.trim().split('/');
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
            Ok(_) => print_output_result(runtime.progress_story(), runtime),
            Err(error) => print_runtime_error(error, runtime),
          }
        }
      } else if let Ok(choice) = str.parse::<usize>() {
        if choice == 0 {
          println!("invalid option");
        }
        print_output_result(runtime.pick_choice(choice - 1), runtime)
      } else {
        println!("Unkown command: {}", str);
      }
    }
  }
}
fn print_all_variables(runtime: &Runtime) {
  let mut variables = Vec::default();

  for variable in runtime.database.config.variables.keys() {
    variables.push(variable.clone());
  }

  variables.sort();
  print_variables(&variables, runtime);
}

fn print_variables(variables: &Vec<String>, runtime: &Runtime) {
  for variable in variables {
    print_variable(variable, runtime);
  }
}

fn print_changed_variables(changed_variables: &Vec<VariableChange>) {
  for variable in changed_variables {
    println!("{} = {}", variable.variable, variable.new_value);
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

fn print_output(output: Output) {
  for block in output.blocks {
    print_block(block);
  }
  println!("{}", output.text);
  print_choices(output.choices);
}

fn print_block(block: Block) {
  //println!("Script:{:?}", block.script);
  print_chance(block.chance);
  print_changed_variables(&block.changed_variables);
  if !block.tags.is_empty() {
    println!("Tags:{:?}", block.tags);
  }

  if !block.functions.is_empty() {
    println!("Functions:{:?}", block.tags);
  }
}

fn print_chance(chance: Chance) {
  match chance {
    Chance::None => {}
    Chance::Probability(value) => {
      println!("🎲 ({}%)", value)
    }
    Chance::Frequency {
      value,
      total_frequency,
    } => {
      println!("🎲 ({}/{})", value, total_frequency)
    }
  }
}

fn print_runtime_error(error: RuntimeError, runtime: &Runtime) {
  match error {
    RuntimeError::WaitingForChoice(_) => {
      println!("Make a choice:\n");
      print_output(runtime.current().unwrap());
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
      print_output(runtime.current().unwrap());
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

fn print_output_result(result: Result<Output, RuntimeError>, runtime: &Runtime) {
  match result {
    Ok(output) => print_output(output),
    Err(error) => print_runtime_error(error, runtime),
  }
}
