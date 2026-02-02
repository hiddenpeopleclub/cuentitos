use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

use crate::PathBuf;
use crate::TestCase;

#[derive(Debug, Clone)]
pub enum TestResult {
    Pass,
    Fail {
        expected: Option<String>,
        actual: String,
    },
}

pub struct TestRunner {
    runtime_path: PathBuf,
}

impl TestRunner {
    pub fn from_path(path: PathBuf) -> Self {
        TestRunner { runtime_path: path }
    }

    pub fn run(&self, test_case: TestCase) -> TestResult {
        // Create a temporary file named after the test case to avoid race conditions
        let temp_filename = format!(
            "{}.cuentitos",
            test_case
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        );
        let mut file = File::create(&temp_filename).unwrap();
        writeln!(file, "{}", test_case.script).unwrap();

        let input_commands = test_case.input.split("\n").collect::<Vec<&str>>().join(",");

        // Run the runtime with the script file and the input from the test case
        let result = match Command::new(&self.runtime_path)
            .args(["run", &temp_filename, &input_commands])
            .output()
        {
            Ok(result) => {
                let output = String::from_utf8(result.stdout.clone()).unwrap_or_default();
                let output_trimmed = output.trim_end_matches(&['\r', '\n'][..]);
                let expected_trimmed = test_case.result.trim_end_matches(&['\r', '\n'][..]);

                if expected_trimmed == output_trimmed {
                    TestResult::Pass
                } else {
                    TestResult::Fail {
                        expected: Some(test_case.result),
                        actual: output,
                    }
                }
            }
            Err(err) => {
                dbg!(err);
                TestResult::Fail {
                    expected: None,
                    actual: "Error running test".to_string(),
                }
            }
        };

        // Remove temporary file
        std::fs::remove_file(&temp_filename).unwrap();

        result
    }
}
