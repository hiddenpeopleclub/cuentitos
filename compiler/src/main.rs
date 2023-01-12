use clap::Parser;
use cuentitos_compiler::compile;

#[derive(Parser,Debug)]
struct Args {
    #[clap(default_value=".")]
    source_path: std::path::PathBuf,
    #[clap(default_value="./cuentitos.db")]
    build_path: std::path::PathBuf,
}

fn main() {
  let args = Args::parse();

  let Args { source_path, build_path } = args;
  {
    let result = compile(&source_path, &build_path).unwrap();

    println!();
    println!("Parse result for events in '{}/events'", source_path.display());
    println!();

    for (id, event) in result.events {
      match event {
        Ok(_) => println!("  ✔️  {}", id),
        Err(error) => println!("  ❌  {}: {}", id, error)
      }
    }

    println!();

  }
}
