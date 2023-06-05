use clap::{Parser, Subcommand};
use std::path::Path;

mod console;
use console::Console;

#[derive(Parser, Debug)]
#[command(name = "cuentitos")]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  Compile {
    #[clap(default_value = ".")]
    source_path: std::path::PathBuf,
    #[clap(default_value = "./build/cuentitos.db")]
    build_path: std::path::PathBuf,
  },

  Run {
    #[clap(default_value = "./cuentitos.db")]
    source_path: std::path::PathBuf,
  },
}

fn compile<T, U>(source_path: T, destination_path: U)
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  palabritas_compiler::compile(&source_path, destination_path);
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Some(Commands::Compile {
      source_path,
      build_path,
    }) => {
      compile(source_path, build_path);
    }
    Some(Commands::Run { source_path }) => {
      Console::start(source_path);
    }
    None => {}
  }
}
