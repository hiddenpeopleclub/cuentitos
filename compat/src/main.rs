use glob::{glob, GlobError};
use std::path::PathBuf;
use clap::Parser;
/// Simple program to greet a person

#[derive(Parser, Debug)]
struct Args {
    /// Runtime path
    // #[arg(short, long)]
    runtime: PathBuf,

    /// Compatibility Tests path
    // #[arg(short, long, default_value_t = 1)]
    compatibility_tests: String,

}


fn main() {
    let args = Args::parse();

    let compatibility_tests = get_compatibility_tests(args.compatibility_tests);

    // println!("Runtime {:?}, {:?}", args.runtime, args.compatibility_tests);

    dbg!(compatibility_tests);
}


fn get_compatibility_tests(path: String) -> Vec<Result<PathBuf, GlobError>>
{
    glob(&path).unwrap().collect()
}
