use crate::test_runner::TestResult;
use crate::test_runner::TestRunner;
use crate::test_case::TestCase;
use glob::{glob, GlobError};
use std::path::PathBuf;
use clap::Parser;

mod test_runner;
mod test_case;

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

  // Check that the runtime exists
  if !args.runtime.exists() {
    eprintln!("Error: Runtime path does not exist");
    return;
  }

  // Check that the runtime is a file
  if !args.runtime.is_file() {
    eprintln!("Error: Runtime path is not a file");
    return;
  }

  let compatibility_tests = get_compatibility_tests(args.compatibility_tests);

  // Check that there are compatibility tests
  if compatibility_tests.len() == 0 {
    eprintln!("Error: No compatibility tests found");
    return;
  }
  let runner = TestRunner::from_path(args.runtime);

  let mut failed_tests = vec![];

  for test in compatibility_tests {
    match test {
      Ok(path) => {
        // Load test content
        let content = std::fs::read_to_string(path);
        match content {
          Ok(content) => {
            let test_case = TestCase::from_string(content);

            match runner.run(test_case.clone()) {
              TestResult::Pass => print!("."),
              TestResult::Fail(reason) => {
                print!("F");
                failed_tests.push((test_case, reason));
              }
            };
          },
          Err(e) => {
            eprintln!("Error: {:?}", e);
          }
        }
      },
      Err(e) => {
        eprintln!("Error: {:?}", e);
      }
    }
  }

  for (test, reason) in failed_tests {
    eprintln!("Test failed: {}", test.name);
    eprintln!("{}", reason);
  }
}


fn get_compatibility_tests(path: String) -> Vec<Result<PathBuf, GlobError>>
{
    glob(&path).unwrap().collect()
}
