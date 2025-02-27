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
        // Create a temporary file to hold the test case
        let mut file = File::create("test.cuentitos").unwrap();
        writeln!(file, "{}", test_case.script).unwrap();

        let input_commands = test_case.input.split("\n").collect::<Vec<&str>>().join(",");

        // Run the runtime with the script file and the input from the test case
        let result = match Command::new(&self.runtime_path)
            .args(["run", "test.cuentitos", &input_commands])
            .output()
        {
            Ok(result) => {
                let mut output = result.stdout.clone();
                output.pop();
                output.pop();

                let mut expected = test_case.result.clone().into_bytes();
                expected.pop();

                if expected == output {
                    TestResult::Pass
                } else {
                    TestResult::Fail {
                        expected: Some(test_case.result),
                        actual: String::from_utf8(result.stdout).unwrap(),
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
        std::fs::remove_file("test.cuentitos").unwrap();

        result
    }
}
