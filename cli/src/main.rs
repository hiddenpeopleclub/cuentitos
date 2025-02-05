use std::path::PathBuf;
use clap::{Parser, Subcommand};
/// Simple program to greet a person

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// does testing things
  Run {
    /// lists test values
    script_path: PathBuf,
    input_string: Option<String>
  },
}

fn main() {
  let cli = Args::parse();

  match cli.command {
    Commands::Run { script_path, input_string } => {

      // Read the script file
      let script = std::fs::read_to_string(script_path).unwrap();

      // Parse it
      let database = cuentitos_parser::parse(&script).unwrap();

      // Run in runtime
      let mut runtime = cuentitos_runtime::Runtime::new(database);

      runtime.run();

      render_current_block(&runtime);
      if let Some(input) = input_string {
        if input != "" {
          input.split(",").for_each(|input| {
            process_input(input, &mut runtime);
            render_current_block(&runtime);
          });
        }
      }
      if runtime.has_ended() {
        println!("END");
        runtime.stop();
      } else {
        println!("DID NOT END");
      }
    }
  }
}

fn process_input(input: &str, runtime: &mut cuentitos_runtime::Runtime) {
  match input {
    "n" => {
      if runtime.can_continue() {
        runtime.step();
      } else {
        panic!("TODO ADR: Input Can't Continue");
      }
    }
    "q" => {
      runtime.stop();
    } ,
    &_ => {}
  }
}

fn render_current_block(runtime: &cuentitos_runtime::Runtime) {
  if let Some(block) = runtime.current_block() {
    match block {
      cuentitos_common::Block::String(id) => {
        println!("{}", runtime.database.strings[id]);
      }
    }
  } else {
    println!("END");
  }
}
