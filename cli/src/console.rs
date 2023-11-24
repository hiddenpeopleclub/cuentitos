use cuentitos_runtime::*;
use palabritas::parse_modifier_str;
use rustyline::{error::ReadlineError, history::FileHistory};
use rustyline::{DefaultEditor, Editor};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct Console {}
impl Console {
  fn prompt(
    rl: &mut Editor<(), FileHistory>,
    section: &Option<Section>,
  ) -> Result<String, ReadlineError> {
    let mut prompt_str = String::from("\n");

    if let Some(section) = section {
      prompt_str.push_str(&format!("{}", section));
    }

    prompt_str.push_str(" > ");

    rl.readline(&prompt_str)
  }

  pub fn start<T>(path: T)
  where
    T: AsRef<Path>,
  {
    let mut runtime = Self::load_runtime(path);

    let mut rl = DefaultEditor::new().unwrap();
    rl.load_history("history.txt").unwrap_or_default();

    loop {
      let read_line = Self::prompt(&mut rl, runtime.get_current_section());
      match Self::process_line(read_line, &mut rl, &mut runtime) {
        Some(message) => println!("{}", message),
        None => break,
      }
    }

    rl.save_history("history.txt").unwrap();
  }

  fn load_runtime<T>(path: T) -> Runtime
  where
    T: AsRef<Path>,
  {
    let mut f = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    let file = cuentitos_runtime::Database::from_u8(&buffer).unwrap();
    Runtime::new(file)
  }

  fn process_line(
    line: Result<String, ReadlineError>,
    rl: &mut Editor<(), FileHistory>,
    runtime: &mut Runtime,
  ) -> Option<String> {
    match line {
      Ok(line) => {
        if line.to_lowercase() == *"q" {
          return None;
        }
        rl.add_history_entry(line.as_str()).unwrap();

        let mut input = line.split_whitespace();
        let command: &str = input.next().unwrap_or_default();
        let mut parameters = Vec::default();
        for parameter in input {
          parameters.push(parameter);
        }
        Some(run_command(command, parameters, runtime))
      }
      Err(ReadlineError::Interrupted) => None,
      Err(ReadlineError::Eof) => None,
      Err(err) => {
        println!("{:?}", err);
        None
      }
    }
  }
}

fn run_command(command: &str, parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  match command {
    "" => progress_story(runtime),
    "next" | "n" => next_block(runtime),
    "sections" => sections_command(parameters, runtime),
    "?" => state_command(parameters, runtime),
    "variables" => variables_command(parameters, runtime),
    "set" => set_command(parameters, runtime),
    "->" => divert(parameters, runtime),
    "<->" => boomerang_divert(parameters, runtime),
    "skip" | "s" => skip(runtime),
    "skip-all" => skip_all(runtime),
    "reset" => reset_command(parameters, runtime),
    "rewind" => rewind_command(parameters, runtime),
    "rewind_to_choice" => rewind_to_choice_command(runtime),
    "rewind_to" => rewind_to(parameters, runtime),
    str => {
      if str.starts_with("->") {
        let substr: String = str.chars().skip(2).collect();
        divert(vec![&substr], runtime)
      } else if str.starts_with("<->") {
        let substr: String = str.chars().skip(3).collect();
        boomerang_divert(vec![&substr], runtime)
      } else if let Ok(choice) = str.parse::<usize>() {
        pick_choice(choice, runtime)
      } else {
        format!("Unkown command: {}", str)
      }
    }
  }
}

fn pick_choice(choice: usize, runtime: &mut Runtime) -> String {
  if choice == 0 {
    "Invalid option".to_string()
  } else {
    match runtime.pick_choice(choice - 1) {
      Ok(output) => get_output_string(output, runtime),
      Err(error) => get_runtime_error_string(error, runtime),
    }
  }
}
fn variables_command(parameters: Vec<&str>, runtime: &Runtime) -> String {
  let mut variables = Vec::default();

  for variable in runtime.database.config.variables.keys() {
    variables.push(variable.clone());
  }

  variables.sort();
  let variables_str = get_variables_string(&variables, runtime);
  let pattern = parameters_to_pattern(parameters);
  grep(&pattern, &variables_str)
}

fn get_variables_string(variables: &Vec<String>, runtime: &Runtime) -> String {
  let mut variables_string = String::default();
  for variable in variables {
    variables_string += &format!("{}\n", get_variable_string(variable, runtime));
  }
  variables_string.to_string()
}

fn get_output_string(output: Output, runtime: &Runtime) -> String {
  let mut output_string = String::default();
  for block in output.blocks {
    let block_output = get_block_string(block, runtime);
    if !block_output.is_empty() {
      output_string = output_string + &block_output + "\n";
    }
  }

  output_string = output_string.trim_end().to_string();

  let choices_string = get_choices_string(output.choices);
  if !choices_string.is_empty() {
    output_string += &format!("\n{}", choices_string);
  }
  output_string
}

fn get_block_string(block: Block, runtime: &Runtime) -> String {
  let settings = block.get_settings();
  let mut block_string = get_variables_string(&settings.changed_variables, runtime);
  if !settings.tags.is_empty() {
    block_string += &format!("Tags:{:?}\n", settings.tags);
  }

  if !settings.functions.is_empty() {
    block_string += &format!("Functions:{:?}\n", settings.functions);
  }

  match &block {
    Block::Text {
      text,
      i18n_id: _,
      settings,
    } => {
      let chance = get_change_string(&settings.chance);
      format!("{}{}{}", block_string, chance, text)
    }
    Block::Bucket {
      name: Some(name),
      settings,
    } => {
      let chance = get_change_string(&settings.chance);
      format!("{}{}Entered bucket '{}'\n", block_string, chance, name)
    }
    Block::Section { settings } => {
      let chance = get_change_string(&settings.chance);
      format!(
        "{}{}Entered section '{}'\n",
        block_string,
        chance,
        settings.section.clone().unwrap()
      )
    }
    _ => block_string,
  }
  .trim_end()
  .to_string()
}

fn get_change_string(chance: &Chance) -> String {
  match chance {
    Chance::None => String::default(),
    Chance::Probability(value) => {
      format!("🎲 ({}%) ", value)
    }
    Chance::Frequency {
      value,
      total_frequency,
    } => {
      format!("🎲 ({}/{}) ", value, total_frequency)
    }
  }
}

fn get_runtime_error_string(error: RuntimeError, runtime: &mut Runtime) -> String {
  match error {
    RuntimeError::StoryFinished => {
      runtime.reset_story();
      format!("{}", error)
    }
    RuntimeError::WaitingForChoice(_) => format!(
      "Make a choice:\n\n{}",
      get_output_string(runtime.current().unwrap(), runtime)
    ),

    RuntimeError::InvalidChoice {
      total_choices,
      choice_picked,
    } => {
      format!(
        "Can't pick {}, because there's only {} options\nMake a choice:\n\n{}",
        choice_picked + 1,
        total_choices,
        get_output_string(runtime.current().unwrap(), runtime)
      )
    }
    _ => {
      format!("{}", error)
    }
  }
}

fn get_choices_string(choices: Vec<String>) -> String {
  let mut choices_string = String::default();
  for (i, choice) in choices.iter().enumerate() {
    choices_string += &format!("  ({}){}\n", i + 1, choice);
  }
  choices_string
}

fn progress_story(runtime: &mut Runtime) -> String {
  match runtime.progress_story() {
    Ok(output) => get_output_string(output, runtime),
    Err(error) => get_runtime_error_string(error, runtime),
  }
}

fn next_block(runtime: &mut Runtime) -> String {
  match runtime.next_block() {
    Ok(output) => get_output_string(output, runtime),
    Err(error) => get_runtime_error_string(error, runtime),
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
  final_string.trim().to_string()
}

fn sections_command(parameters: Vec<&str>, runtime: &Runtime) -> String {
  println!("Sections:");
  let mut string = String::default();
  let mut sections = Vec::default();
  for section in runtime.database.sections.keys() {
    sections.push(section.to_string());
  }
  sections.sort();

  for section in sections {
    string += &format!("{}\n", section);
  }

  let pattern = parameters_to_pattern(parameters);
  grep(&pattern, &string)
}

fn parameters_to_pattern(parameters: Vec<&str>) -> String {
  let mut pattern = String::default();
  for parameter in parameters {
    pattern.push_str(&(parameter.to_string() + " "));
  }
  pattern.trim().to_string()
}

fn rewind_command(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  if parameters.len() > 1 {
    return "Invalid parameters".to_string();
  }

  if !parameters.is_empty() {
    if let Ok(rewind_count) = parameters[0].parse() {
      for _i in 0..rewind_count {
        runtime.rewind().unwrap();
      }
    } else {
      return "Invalid parameters".to_string();
    }
  } else {
    runtime.rewind().unwrap();
  }

  match runtime.current() {
    Ok(current) => get_output_string(current, runtime),
    Err(err) => get_runtime_error_string(err, runtime),
  }
}

fn rewind_to_choice_command(runtime: &mut Runtime) -> String {
  runtime.rewind_to_choice().unwrap();

  match runtime.current() {
    Ok(current) => get_output_string(current, runtime),
    Err(err) => get_runtime_error_string(err, runtime),
  }
}

fn rewind_to(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  if parameters.len() > 1 {
    return "Invalid parameters".to_string();
  }

  if !parameters.is_empty() {
    if let Ok(index) = parameters[0].parse() {
      runtime.rewind_to(index).unwrap();
    } else {
      return "Invalid parameters".to_string();
    }
  } else {
    return "Missing parameter index".to_string();
  }

  match runtime.current() {
    Ok(current) => get_output_string(current, runtime),
    Err(err) => get_runtime_error_string(err, runtime),
  }
}

fn reset_command(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  if parameters.is_empty()
    || parameters.contains(&"all")
    || (parameters.contains(&"story") && parameters.contains(&"reset"))
  {
    runtime.reset_all();
    "State and story reset".to_string()
  } else if parameters.contains(&"state") {
    runtime.reset_state();
    "State reset".to_string()
  } else if parameters.contains(&"story") {
    runtime.reset_story();
    "Story reset".to_string()
  } else {
    format!("Unknown parameters: '{:?}'", parameters)
  }
}

fn state_command(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  let mut string = String::default();
  if !runtime.get_current_block_stack().is_empty() {
    match runtime.current() {
      Ok(result) => {
        string.push_str("Current Block:\n");
        string.push_str(&format!("  Text: {}\n", result.text));
        string.push_str(&format!(
          "  Script: {}\n\n",
          result.blocks.last().unwrap().get_settings().script
        ));
      }
      Err(error) => string.push_str(&get_runtime_error_string(error, runtime)),
    }
    match runtime.peek_next() {
      Ok(result) => {
        string.push_str("Next Block:\n");
        string.push_str(&format!("  Text: {}\n", result.text));
        string.push_str(&format!(
          "  Script: {}\n\n",
          result.blocks.last().unwrap().get_settings().script
        ));
      }
      Err(error) => string.push_str(&get_runtime_error_string(error, runtime)),
    }
  }
  string.push_str("Variables: \n");
  string.push_str(&variables_command(vec![], runtime));

  let pattern = parameters_to_pattern(parameters);
  grep(&pattern, &string)
}

fn get_variable_string(variable: &String, runtime: &Runtime) -> String {
  let mut variable_string = String::default();
  for (runtime_variable, kind) in &runtime.database.config.variables {
    if runtime_variable == variable {
      match kind {
        VariableKind::Integer => {
          let int: i32 = runtime.get_variable(variable).unwrap();
          variable_string += &format!("{} = {}", variable, int);
        }
        VariableKind::Float => {
          let float: f32 = runtime.get_variable(variable).unwrap();
          variable_string += &format!("{} = {}", variable, float);
        }
        VariableKind::Bool => {
          let bool: bool = runtime.get_variable(variable).unwrap();
          variable_string += &format!("{} = {}", variable, bool);
        }
        _ => {
          let string: String = runtime.get_variable(variable).unwrap();
          variable_string += &format!("{} = {}", variable, string);
        }
      }
      return variable_string;
    }
  }
  format!("Variable {} doesn't exist", variable)
}

fn set_command(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  let mut modifier_str = "set".to_string();
  for parameter in parameters {
    modifier_str += &format!(" {}", parameter);
  }
  match parse_modifier_str(&modifier_str, &runtime.database.config) {
    Ok(modifier) => match runtime.apply_modifier(&modifier) {
      Ok(_) => get_variable_string(&modifier.variable, runtime),
      Err(error) => format!("{}", error),
    },
    Err(error) => format!("{}", error),
  }
}

fn skip(runtime: &mut Runtime) -> String {
  match runtime.skip() {
    Ok(output) => {
      let mut output_string = output.text;
      let choices_string = get_choices_string(output.choices);
      if !choices_string.is_empty() {
        output_string += &format!("\n{}", choices_string);
      }
      output_string
    }
    Err(error) => get_runtime_error_string(error, runtime),
  }
}

fn skip_all(runtime: &mut Runtime) -> String {
  match runtime.skip_all() {
    Ok(output) => {
      let mut output_string = output.text;
      let choices_string = get_choices_string(output.choices);
      if !choices_string.is_empty() {
        output_string += &format!("\n{}", choices_string);
      }
      output_string
    }
    Err(error) => get_runtime_error_string(error, runtime),
  }
}

fn boomerang_divert(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  if parameters.is_empty() {
    return "Provide a section".to_string();
  }

  let section = Section::from_str(parameters[0]).unwrap();

  match runtime.boomerang_divert(&section) {
    Ok(blocks) => {
      let mut string = String::default();
      for block in blocks {
        string += &format!("{}\n", get_block_string(block, runtime));
      }
      string = string.trim().to_string() + "\n\n";
      string += &progress_story(runtime);
      string
    }
    Err(error) => get_runtime_error_string(error, runtime),
  }
}

fn divert(parameters: Vec<&str>, runtime: &mut Runtime) -> String {
  if parameters.is_empty() {
    return "Provide a section".to_string();
  }

  let section = Section::from_str(parameters[0]).unwrap();

  match runtime.divert(&section) {
    Ok(blocks) => {
      let mut string = String::default();
      for block in blocks {
        string += &format!("{}\n", get_block_string(block, runtime));
      }
      string = string.trim().to_string() + "\n\n";
      string += &progress_story(runtime);
      string
    }
    Err(error) => get_runtime_error_string(error, runtime),
  }
}

#[cfg(test)]
mod test {

  use crate::console::*;

  #[test]
  fn reset_command() {
    let mut rl = DefaultEditor::new().unwrap();
    let mut runtime = Runtime::default();
    runtime.game_state.uniques_played.push(0);
    runtime
      .game_state
      .variables
      .insert("variable".to_string(), "true".to_string());
    assert_ne!(runtime.game_state, GameState::default());

    let str_found =
      Console::process_line(Ok("reset state".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(str_found, "State reset".to_string());
    assert_eq!(runtime.game_state, GameState::default());

    runtime.game_state.block_stack.push(BlockStackData {
      id: 0,
      chance: Chance::None,
    });
    runtime.game_state.choices.push(1);
    runtime.game_state.section = Some(Section::default());

    assert!(!runtime.get_current_block_stack().is_empty());
    assert!(!runtime.get_current_choices().is_empty());
    assert!(runtime.get_current_section().is_some());
    let str_found =
      Console::process_line(Ok("reset story".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(str_found, "Story reset".to_string());
    assert!(runtime.get_current_block_stack().is_empty());
    assert!(runtime.get_current_choices().is_empty());
    assert!(runtime.get_current_section().is_none());
    runtime.game_state.uniques_played.push(0);
    runtime
      .game_state
      .variables
      .insert("variable".to_string(), "true".to_string());
    runtime.game_state.block_stack.push(BlockStackData {
      id: 0,
      chance: Chance::None,
    });
    runtime.game_state.choices.push(1);
    runtime.game_state.section = Some(Section::default());

    assert_ne!(runtime.game_state, GameState::default());
    assert!(!runtime.get_current_block_stack().is_empty());
    assert!(!runtime.get_current_choices().is_empty());
    assert!(runtime.get_current_section().is_some());

    let str_found = Console::process_line(Ok("reset".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(str_found, "State and story reset".to_string());
    assert_eq!(runtime.game_state, GameState::default());
    assert!(runtime.get_current_block_stack().is_empty());
    assert!(runtime.get_current_choices().is_empty());
    assert!(runtime.get_current_section().is_none());
  }

  #[test]
  fn q_command() {
    let mut runtime = Runtime::default();
    let mut rl = DefaultEditor::new().unwrap();
    assert_eq!(
      Console::process_line(Ok("q".to_string()), &mut rl, &mut runtime),
      None
    )
  }

  #[test]
  fn progress_story_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "You've just arrived in the bustling city, full of excitement and anticipation for your new job.";
    let str_found = Console::process_line(Ok("".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    runtime.reset_all();
    runtime.database.config.story_progress_style = StoryProgressStyle::Skip;

    let expected_str = "You've just arrived in the bustling city, full of excitement and anticipation for your new job.\nThe skyline reaches for the clouds, and the sounds of traffic and people surround you.\nAs you take your first steps in this urban jungle, you feel a mix of emotions, hoping to find your place in this new environment.\n  (1)I take a walk through a nearby park to relax and acclimate to the city.\n  (2)I visit a popular street market to experience the city's unique flavors and energy.\n";
    let str_found = Console::process_line(Ok("".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn sections_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "second_day\nsecond_day/farmers_market\nsecond_day/museum";
    let str_found =
      Console::process_line(Ok("sections".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn question_mark_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "Variables: \nenergy = 0\nitem = tea\ntime = 0\ntime_of_day = morning";
    let str_found = Console::process_line(Ok("?".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    runtime.progress_story().unwrap();
    let expected_str = "Current Block:\n  Text: You've just arrived in the bustling city, full of excitement and anticipation for your new job.\n  Script: .\\examples\\story-example.cuentitos:1:1\n\nNext Block:\n  Text: The skyline reaches for the clouds, and the sounds of traffic and people surround you.\n  Script: .\\examples\\story-example.cuentitos:2:1\n\nVariables: \nenergy = 0\nitem = tea\ntime = 0\ntime_of_day = morning";
    let str_found = Console::process_line(Ok("?".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn variables_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "energy = 0\nitem = tea\ntime = 0\ntime_of_day = morning";
    let str_found =
      Console::process_line(Ok("variables".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn set_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "energy = 10";
    let str_found =
      Console::process_line(Ok("set energy 10".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    let energy_value: i32 = runtime.get_variable("energy").unwrap();
    assert_eq!(energy_value, 10);
  }

  #[test]
  fn divert_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "Entered section 'second_day'\n\nYou wake up feeling refreshed. Let's see what this day brings.\n  (1)Explore a museum\n  (2)Go to the Farmer's Market\n";
    let str_found =
      Console::process_line(Ok("-> second_day".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    let section_found = runtime.get_current_section().clone().unwrap();
    let expected_section = Section {
      name: "second_day".to_string(),
      parent: None,
    };
    assert_eq!(section_found, expected_section);

    let expected_stack: Vec<BlockStackData> = vec![
      BlockStackData {
        id: 10,
        chance: Chance::None,
      },
      BlockStackData {
        id: 17,
        chance: Chance::None,
      },
    ];
    let stack_found = runtime.get_current_block_stack().clone();
    assert_eq!(expected_stack, stack_found);

    let expected_str = "Entered section 'second_day/museum'\n\nYou get to the museum door. You watch through the window. It seems crowded.";
    let str_found =
      Console::process_line(Ok("->second_day/museum".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    let section_found = runtime.get_current_section().clone().unwrap();
    let expected_section = Section {
      name: "museum".to_string(),
      parent: Some(Box::new(expected_section)),
    };
    assert_eq!(section_found, expected_section);

    let expected_stack: Vec<BlockStackData> = vec![
      BlockStackData {
        id: 18,
        chance: Chance::None,
      },
      BlockStackData {
        id: 34,
        chance: Chance::None,
      },
    ];
    let stack_found = runtime.get_current_block_stack().clone();
    assert_eq!(expected_stack, stack_found);
  }

  #[test]
  fn boomerang_divert_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "Entered section 'second_day'\n\nYou wake up feeling refreshed. Let's see what this day brings.\n  (1)Explore a museum\n  (2)Go to the Farmer's Market\n";
    let str_found =
      Console::process_line(Ok("<-> second_day".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    let section_found = runtime.get_current_section().clone().unwrap();
    let expected_section = Section {
      name: "second_day".to_string(),
      parent: None,
    };
    assert_eq!(section_found, expected_section);

    let expected_stack: Vec<BlockStackData> = vec![
      BlockStackData {
        id: 10,
        chance: Chance::None,
      },
      BlockStackData {
        id: 17,
        chance: Chance::None,
      },
    ];
    let stack_found = runtime.get_current_block_stack().clone();
    assert_eq!(expected_stack, stack_found);

    let expected_str = "Entered section 'second_day/museum'\n\nYou get to the museum door. You watch through the window. It seems crowded.";
    let str_found = Console::process_line(
      Ok("<->second_day/museum".to_string()),
      &mut rl,
      &mut runtime,
    )
    .unwrap();
    assert_eq!(expected_str, &str_found);

    let section_found = runtime.get_current_section().clone().unwrap();
    let expected_section = Section {
      name: "museum".to_string(),
      parent: Some(Box::new(expected_section)),
    };
    assert_eq!(section_found, expected_section);

    let expected_stack: Vec<BlockStackData> = vec![
      BlockStackData {
        id: 10,
        chance: Chance::None,
      },
      BlockStackData {
        id: 17,
        chance: Chance::None,
      },
      BlockStackData {
        id: 18,
        chance: Chance::None,
      },
      BlockStackData {
        id: 34,
        chance: Chance::None,
      },
    ];
    let stack_found = runtime.get_current_block_stack().clone();
    assert_eq!(expected_stack, stack_found);
  }

  #[test]
  fn skip_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "You've just arrived in the bustling city, full of excitement and anticipation for your new job.\nThe skyline reaches for the clouds, and the sounds of traffic and people surround you.\nAs you take your first steps in this urban jungle, you feel a mix of emotions, hoping to find your place in this new environment.\n  (1)I take a walk through a nearby park to relax and acclimate to the city.\n  (2)I visit a popular street market to experience the city's unique flavors and energy.\n";
    let str_found = Console::process_line(Ok("skip".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    runtime.reset_all();

    let str_found = Console::process_line(Ok("s".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn skip_all_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "As you take your first steps in this urban jungle, you feel a mix of emotions, hoping to find your place in this new environment.\n  (1)I take a walk through a nearby park to relax and acclimate to the city.\n  (2)I visit a popular street market to experience the city's unique flavors and energy.\n";
    let str_found =
      Console::process_line(Ok("skip-all".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn next_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    let mut rl = DefaultEditor::new().unwrap();

    let expected_str = "You've just arrived in the bustling city, full of excitement and anticipation for your new job.";
    let str_found = Console::process_line(Ok("n".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    runtime.reset_all();

    let str_found = Console::process_line(Ok("next".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn rewind_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    runtime.database.config.keep_history = true;
    runtime.next_block().unwrap();
    runtime.next_block().unwrap();

    let mut rl = DefaultEditor::new().unwrap();
    let expected_str = "You've just arrived in the bustling city, full of excitement and anticipation for your new job.";

    let str_found = Console::process_line(Ok("rewind".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);

    runtime.next_block().unwrap();
    runtime.next_block().unwrap();

    let str_found =
      Console::process_line(Ok("rewind 2".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn rewind_to_choice_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    runtime.database.config.keep_history = true;
    runtime.skip().unwrap();
    runtime.pick_choice(0).unwrap();

    let mut rl = DefaultEditor::new().unwrap();
    let expected_str = "As you take your first steps in this urban jungle, you feel a mix of emotions, hoping to find your place in this new environment.\n  (1)I take a walk through a nearby park to relax and acclimate to the city.\n  (2)I visit a popular street market to experience the city's unique flavors and energy.\n";

    let str_found =
      Console::process_line(Ok("rewind_to_choice".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }

  #[test]
  fn rewind_to_command() {
    let mut runtime = Console::load_runtime("./fixtures/script");
    runtime.database.config.keep_history = true;
    runtime.next_block().unwrap();
    runtime.next_block().unwrap();
    runtime.next_block().unwrap();

    let mut rl = DefaultEditor::new().unwrap();
    let expected_str =
      "The skyline reaches for the clouds, and the sounds of traffic and people surround you.";

    let str_found =
      Console::process_line(Ok("rewind_to 2".to_string()), &mut rl, &mut runtime).unwrap();
    assert_eq!(expected_str, &str_found);
  }
}
