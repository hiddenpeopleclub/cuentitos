use clap::{Parser, Subcommand};
use cuentitos_compiler::compile;

mod console;
use console::Console;


#[derive(Parser, Debug)]
#[command(name = "cuentitos")]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {
      #[clap(default_value = ".")]
      source_path: std::path::PathBuf,
      #[clap(default_value = "./cuentitos.db")]
      build_path: std::path::PathBuf,
    },
    
    Run {
      #[clap(default_value = "./cuentitos.db")]
      source_path: std::path::PathBuf,
    }
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Some(Commands::Compile { source_path, build_path }) => {
      let result = compile(&source_path, &build_path).unwrap();

      println!();
      println!(
        "Parse result for events in '{}/events'",
        source_path.display()
      );
      println!();

      for (id, event) in result.events {
        match event {
          Ok(_) => println!("  ✔️  {}", id),
          Err(error) => println!("  ❌  {}: {}", id, error),
        }
      }
    },
    Some(Commands::Run { source_path }) => {
      Console::start(source_path);
    },
    None => {}
  }



  //   println!();
  // }
}
