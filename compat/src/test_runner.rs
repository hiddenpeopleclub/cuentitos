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

struct TempFileGuard {
    path: PathBuf,
    dir: PathBuf,
}

impl TempFileGuard {
    fn new(path: PathBuf, dir: PathBuf) -> Self {
        Self { path, dir }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.dir);
    }
}

impl TestRunner {
    pub fn from_path(path: PathBuf) -> Self {
        TestRunner { runtime_path: path }
    }

    pub fn run(&self, test_case: TestCase) -> TestResult {
        // Create a temporary file in the system temp directory
        let file_stem = test_case
            .path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!(
            "cuentitos-compat-{}-{}",
            std::process::id(),
            unique
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let temp_path = temp_dir.join(format!("{}.cuentitos", file_stem));
        let _temp_guard = TempFileGuard::new(temp_path.clone(), temp_dir);
        let mut file = File::create(&temp_path).unwrap();
        writeln!(file, "{}", test_case.script).unwrap();

        let input_commands = test_case.input.split("\n").collect::<Vec<&str>>().join(",");

        // Run the runtime with the script file and the input from the test case
        let result = match Command::new(&self.runtime_path)
            .arg("run")
            .arg(&temp_path)
            .arg(&input_commands)
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

        result
    }
}
