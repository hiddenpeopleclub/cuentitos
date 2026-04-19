use crate::test_runner::TestResult;
use crate::test_runner::TestRunner;
use clap::Parser;
use colored::Colorize;
use cuentitos_common::test_case::TestCase;
use glob::{glob, GlobError};
use std::path::PathBuf;
use std::process::ExitCode;

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

fn main() -> ExitCode {
    let args = Args::parse();

    // Check that the runtime exists
    if !args.runtime.exists() {
        println!("Error: Runtime path does not exist");
        return ExitCode::from(2);
    }

    // Check that the runtime is a file
    if !args.runtime.is_file() {
        println!("Error: Runtime path is not a file");
        return ExitCode::from(2);
    }

    let compatibility_tests = get_compatibility_tests(args.compatibility_tests);

    // Check that there are compatibility tests
    let test_count = compatibility_tests.len();

    if compatibility_tests.is_empty() {
        println!("Error: No compatibility tests found");
        return ExitCode::from(2);
    }
    let runner = TestRunner::from_path(args.runtime);

    let mut failed_tests = vec![];
    let mut pending_tests = vec![];

    for test in compatibility_tests {
        match test {
            Ok(path) => {
                // Load test content
                let content = std::fs::read_to_string(&path);
                match content {
                    Ok(content) => {
                        let test_case = TestCase::from_string(content, &path);

                        if test_case.pending_reason.is_some() {
                            print!("{}", "P".yellow());
                            pending_tests.push(test_case);
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
    let pending_count = pending_tests.len();

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

    if pending_count > 0 {
        println!("{}", "------------------------------------".yellow().bold());
        println!("{}", "Pending tests:".yellow().bold());
        for test in &pending_tests {
            let reason = test.pending_reason.as_deref().unwrap_or("");
            println!(
                "  {} ({}): {}",
                test.name.bold(),
                test.path.to_str().unwrap_or(""),
                reason
            );
        }
    }

    println!("\n");
    println!(
        "{} {} | {} {} | {} {}",
        "Total:".bold(),
        test_count,
        "Failed:".red().bold(),
        failed_count,
        "Pending:".yellow().bold(),
        pending_count
    );

    if failed_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn get_compatibility_tests(path: String) -> Vec<Result<PathBuf, GlobError>> {
    glob(&path).unwrap().collect()
}
