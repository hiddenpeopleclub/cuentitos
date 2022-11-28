use clap::Parser;
use cuentitos_compiler::compile;

#[derive(Parser,Debug)]
struct Args {
    source_path: std::path::PathBuf,
    #[clap(default_value="./cuentitos.db")]
    build_path: std::path::PathBuf,
}

fn main() {
  let args = Args::parse();

  let Args { source_path, build_path } = args;
  {
    let result = compile(source_path, build_path);
    if result.is_err()
    {
      println!("{}",result.unwrap_err());
    }
  }
}
