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
  let mut input = input.split_whitespace();
  let command = input.next().unwrap_or_default();
  let mut parameters = Vec::default();
  for parameter in input {
    parameters.push(parameter.to_string());
  }

  match command {
    "" => print_output_result(runtime.progress_story(), runtime),
    "sections" => {
      print_sections(parameters, runtime);
    }
    "?" => {
      print_state(parameters, runtime);
    }
    "variables" => print_all_variables(runtime),
    "set" => {
      set(parameters, runtime);
    }
    "->" => {
      if parameters.is_empty() {
        println!("Provide a section");
      } else {
        divert(parameters[0].clone(), runtime);
      }
    }
    str => {
      if str.starts_with("->") {
        let substr: String = str.chars().skip(2).collect();
        divert(substr, runtime);
      }
      if let Ok(choice) = str.parse::<usize>() {
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

fn print_output(output: Output, runtime: &Runtime) {
  for block in output.blocks {
    print_block(block, runtime);
  }
  print_choices(output.choices);
}

fn print_block(block: Block, runtime: &Runtime) {
  let settings = block.get_settings();
  print_variables(&settings.changed_variables, runtime);
  if !settings.tags.is_empty() {
    println!("Tags:{:?}\n", settings.tags);
  }

  if !settings.functions.is_empty() {
    println!("Functions:{:?}\n", settings.functions);
  }

  match &block {
    Block::Text { text, settings } => {
      let chance = get_change_string(&settings.chance);
      println!("{}{}", chance, text);
    }
    Block::Bucket {
      name: Some(name),
      settings,
    } => {
      let chance = get_change_string(&settings.chance);
      println!("{}Entered bucket '{}\n'", chance, name);
    }
    Block::Section { name, settings } => {
      let chance = get_change_string(&settings.chance);
      println!("{}Entered section '{}'\n", chance, name);
    }
    Block::Subsection {
      section,
      name,
      settings,
    } => {
      let chance = get_change_string(&settings.chance);
      println!("{}Entered section '{}/{}'\n", chance, section, name);
    }
    _ => {}
  }
}

fn get_change_string(chance: &Chance) -> String {
  match chance {
    Chance::None => String::default(),
    Chance::Probability(value) => {
      format!("🎲 ({}%)", value)
    }
    Chance::Frequency {
      value,
      total_frequency,
    } => {
      format!("🎲 ({}/{})", value, total_frequency)
    }
  }
}

fn print_runtime_error(error: RuntimeError, runtime: &Runtime) {
  match error {
    RuntimeError::WaitingForChoice(_) => {
      println!("Make a choice:\n");
      print_output(runtime.current().unwrap(), runtime);
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
      print_output(runtime.current().unwrap(), runtime);
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
    Ok(output) => print_output(output, runtime),
    Err(error) => print_runtime_error(error, runtime),
  }
}

fn grep(pattern: &str, source: &str) -> String {
  let mut final_string = String::default();

  for line in source.split('\n') {
    if line.to_lowercase().contains(&pattern.to_lowercase()) {
      final_string.push_str(line);
      final_string.push('\n');
    }
  }

  if final_string.is_empty() {
    final_string = "no results".to_string();
  }

  final_string
}

fn print_sections(parameters: Vec<String>, runtime: &Runtime) {
  println!("Sections:");
  let mut string = String::default();
  for section in runtime.database.sections.keys() {
    string.push_str(&(section.to_string() + "\n"));
  }

  let pattern = parameters_to_pattern(parameters);
  println!("{}", grep(&pattern, &string));
}

fn parameters_to_pattern(parameters: Vec<String>) -> String {
  let mut pattern = String::default();
  for parameter in parameters {
    pattern.push_str(&(parameter + " "));
  }
  pattern.trim().to_string()
}

fn print_state(parameters: Vec<String>, runtime: &Runtime) {
  let mut string = String::default();
  if !runtime.block_stack.is_empty() {
    match runtime.current() {
      Ok(result) => {
        string.push_str("Current Block:\n");
        string.push_str(&format!("  Text: {}\n", result.text));
        string.push_str(&format!(
          "  Script: {}\n",
          result.blocks.last().unwrap().get_settings().script
        ));
      }
      Err(error) => print_runtime_error(error, runtime),
    }
    match runtime.peek_next() {
      Ok(result) => {
        string.push_str("Next Block:");
        string.push_str(&format!("  Text: {}\n", result.text));
        string.push_str(&format!(
          "  Script: {}\n",
          result.blocks.last().unwrap().get_settings().script
        ));
      }
      Err(error) => print_runtime_error(error, runtime),
    }
  }
  string.push_str("Variables: \n");
  string.push_str(&get_all_variables_string(runtime));

  let pattern = parameters_to_pattern(parameters);
  println!("{}", grep(&pattern, &string));
}

fn get_all_variables_string(runtime: &Runtime) -> String {
  let mut variables = Vec::default();
  for variable in runtime.database.config.variables.keys() {
    variables.push(variable.clone());
  }

  variables.sort();

  let mut string = String::default();
  for variable in variables {
    if let Some(str) = get_variable_string(&variable, runtime) {
      string.push_str(&(str + "\n"));
    }
  }
  string
}

fn get_variable_string(variable: &String, runtime: &Runtime) -> Option<String> {
  for (runtime_variable, kind) in &runtime.database.config.variables {
    if runtime_variable == variable {
      match kind {
        VariableKind::Integer => {
          let int: i32 = runtime.get_variable(variable).unwrap();
          return Some(format!("{}: {}", variable, int));
        }
        VariableKind::Float => {
          let float: f32 = runtime.get_variable(variable).unwrap();
          return Some(format!("{}: {}", variable, float));
        }
        VariableKind::Bool => {
          let bool: bool = runtime.get_variable(variable).unwrap();
          return Some(format!("{}: {}", variable, bool));
        }
        _ => {
          let string: String = runtime.get_variable(variable).unwrap();
          return Some(format!("{}: {}", variable, string));
        }
      }
    }
  }
  None
}

fn set(parameters: Vec<String>, runtime: &mut Runtime) {
  let mut modifier_str = "set".to_string();
  for parameter in parameters {
    modifier_str += &format!(" {}", parameter);
  }
  match parse_modifier_str(&modifier_str, &runtime.database.config) {
    Ok(modifier) => match runtime.apply_modifier(&modifier) {
      Ok(_) => {
        print_variable(&modifier.variable, runtime);
      }
      Err(error) => println!("{}", error),
    },
    Err(error) => println!("{}", error),
  }
}

fn divert(section: String, runtime: &mut Runtime) {
  let mut splitted = section.split('/');
  let section_name = splitted.next().unwrap().to_string();
  let subsection_name = splitted.next().map(|str| str.to_string());

  let section = Section {
    section_name,
    subsection_name,
  };

  match runtime.divert(&section) {
    Ok(_) => print_output_result(runtime.progress_story(), runtime),
    Err(error) => print_runtime_error(error, runtime),
  }
}
