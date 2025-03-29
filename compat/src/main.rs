use crate::test_runner::TestResult;
use crate::test_runner::TestRunner;
use clap::Parser;
use colored::Colorize;
use cuentitos_common::test_case::TestCase;
use glob::{glob, GlobError};
use std::path::PathBuf;

mod test_runner;

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
        println!("Error: Runtime path does not exist");
        return;
    }

    // Check that the runtime is a file
    if !args.runtime.is_file() {
        println!("Error: Runtime path is not a file");
        return;
    }

    let compatibility_tests = get_compatibility_tests(args.compatibility_tests);

    // Check that there are compatibility tests
    let test_count = compatibility_tests.len();

    if compatibility_tests.is_empty() {
        println!("Error: No compatibility tests found");
        return;
    }
    let runner = TestRunner::from_path(args.runtime);

    let mut failed_tests = vec![];
    let mut disabled_tests = vec![];

    for test in compatibility_tests {
        match test {
            Ok(path) => {
                // Load test content
                let content = std::fs::read_to_string(&path);
                match content {
                    Ok(content) => {
                        let test_case = TestCase::from_string(content, &path);

                        if test_case.disabled {
                            print!("{}", "*".yellow());
                            disabled_tests.push(test_case);
                            continue;
                        }

                        match runner.run(test_case.clone()) {
                            TestResult::Pass => print!("{}", ".".green()),
                            TestResult::Fail { expected, actual } => {
                                print!("{}", "F".red());
                                failed_tests
                                    .push((test_case, TestResult::Fail { expected, actual }));
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    println!("\n");

    let failed_count = failed_tests.len();
    let disabled_count = disabled_tests.len();

    for (test, fail) in failed_tests {
        println!("{}", "------------------------------------".red().bold());
        if let TestResult::Fail { expected, actual } = fail {
            println!("{} {}", "Test failed:".red().bold(), test.name.bold());
            println!("File: {}", test.path.to_str().unwrap());
            println!("\n");

            if let Some(expected) = expected {
                println!("{}", "Expected Result:\n".bold());
                println!("{}", expected.on_green());
                println!("\n");
            }

            println!("{}", "Actual Result:\n".bold());
            println!("{}", actual.on_red());
        }
    }

    if !disabled_tests.is_empty() {
        println!("\n{}", "Disabled Tests:".yellow().bold());
        for test in disabled_tests {
            println!("  {} ({})", test.name.yellow(), test.path.to_str().unwrap());
        }
    }

    println!("\n");
    println!(
        "{} {} | {} {} | {} {}",
        "Total:".bold(),
        test_count,
        "Failed:".red().bold(),
        failed_count,
        "Disabled:".yellow().bold(),
        disabled_count
    );
}

fn get_compatibility_tests(path: String) -> Vec<Result<PathBuf, GlobError>> {
    glob(&path).unwrap().collect()
}
