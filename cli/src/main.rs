use clap::{Parser, Subcommand};
use notify::event::AccessKind::Close;
use notify::event::AccessMode::Write;
use notify::{EventKind, RecursiveMode, Result, Watcher};
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
    source_path: std::path::PathBuf,
    #[clap(default_value = "./build/cuentitos.db")]
    build_path: std::path::PathBuf,
  },

  Run {
    #[clap(default_value = "./cuentitos.db")]
    source_path: std::path::PathBuf,
  },

  Watch {
    #[clap(default_value = ".")]
    source_path: std::path::PathBuf,
    #[clap(default_value = "./cuentitos.db")]
    build_path: std::path::PathBuf,
  },
}

fn compile<T, U>(source_path: T, destination_path: U)
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  cuentitos_compiler::compile(&source_path, destination_path);
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
    Some(Commands::Watch {
      source_path,
      build_path,
    }) => {
      watch(source_path, build_path).unwrap();
    }
    None => {}
  }
}


fn watch<T, U>(source_path: T, destination_path: U) -> Result<()>
where
  T: AsRef<Path>,
  U: AsRef<Path>,
{
  compile(&source_path, &destination_path);
  let source_path_moved = source_path.as_ref().to_path_buf();
  let destination_path = destination_path.as_ref().to_path_buf();

  let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event>| match res {
    Ok(event) => {
      if event.kind == EventKind::Access(Close(Write)) {
        compile(&source_path_moved, &destination_path);
      }
    }
    Err(e) => println!("watch error: {:?}", e),
  })
  .unwrap();

  // Add a path to be watched. All files and directories at that path and
  // below will be monitored for changes.
  let mut listening_paths = source_path.as_ref().to_path_buf();
  listening_paths.push("events");
  watcher
    .watch(&listening_paths, RecursiveMode::Recursive)
    .unwrap();

  let mut listening_paths = source_path.as_ref().to_path_buf();
  listening_paths.push("items");
  if listening_paths.is_dir() {
    watcher
      .watch(&listening_paths, RecursiveMode::Recursive)
      .unwrap();
  }

  loop {}
}