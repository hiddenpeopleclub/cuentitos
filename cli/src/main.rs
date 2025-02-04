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

      while runtime.running() {
        if let Some(input) = input_string {
          input.split(",").for_each(|input| {
            println!("Input string: {}", input);
          });
        }
        break;
      }

      // println!("This is a single line");
      // println!("END");
      // println!("Running script: {:?}", script_path);
      // if let Some(input) = input_string {
      //   println!("Input string: {}", input);
      // }
    }
  }
}
