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

#[derive(Debug)]
#[allow(dead_code)]
pub enum TestError {
    IoError(std::io::Error),
    Utf8Error(std::string::FromUtf8Error),
}

impl From<std::io::Error> for TestError {
    fn from(err: std::io::Error) -> Self {
        TestError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for TestError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        TestError::Utf8Error(err)
    }
}

pub struct TestRunner {
    runtime_path: PathBuf,
    test_file: PathBuf,
}

impl TestRunner {
    pub fn from_path(path: PathBuf) -> Self {
        TestRunner {
            runtime_path: path,
            test_file: PathBuf::from("test.cuentitos"),
        }
    }

    pub fn run(&self, test_case: TestCase) -> Result<TestResult, TestError> {
        // Create a temporary file to hold the test case
        let mut file = File::create(&self.test_file)?;
        writeln!(file, "{}", test_case.script)?;
        drop(file); // Ensure file is closed before running the command

        let input_commands = test_case.input.split("\n").collect::<Vec<&str>>().join(",");

        // Run the runtime with the script file and the input from the test case
        let result = match Command::new(&self.runtime_path)
            .args(["run", self.test_file.to_str().unwrap(), &input_commands])
            .output()
        {
            Ok(result) => {
                let output_str = String::from_utf8(result.stdout)?;
                let mut output_lines: Vec<&str> = output_str.lines().collect();

                // Remove trailing empty lines
                while output_lines.last().map_or(false, |line| line.is_empty()) {
                    output_lines.pop();
                }

                let actual = output_lines.join("\n");
                let expected = test_case.result.trim_end();

                if expected == actual {
                    TestResult::Pass
                } else {
                    TestResult::Fail {
                        expected: Some(test_case.result),
                        actual: output_str,
                    }
                }
            }
            Err(err) => {
                eprintln!("Error running test: {:?}", err);
                // Clean up the file before returning the error
                let _ = std::fs::remove_file(&self.test_file);
                return Err(TestError::IoError(err));
            }
        };

        // Remove temporary file
        if let Err(e) = std::fs::remove_file(&self.test_file) {
            eprintln!("Warning: Failed to remove test file: {:?}", e);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn create_test_case() -> TestCase {
        TestCase {
            name: "Test".to_string(),
            script: "test script".to_string(),
            input: "n".to_string(),
            result: "expected result".to_string(),
            path: PathBuf::from("test.md"),
            disabled: false,
        }
    }

    fn cleanup_dir(dir: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Reset permissions so we can delete it
            if let Ok(metadata) = fs::metadata(dir) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(dir, perms);
            }
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_io_error_on_file_creation() {
        // Clean up from any previous failed runs
        let dir = Path::new("readonly_dir");
        cleanup_dir(dir);

        // Create a read-only directory to trigger file creation error
        fs::create_dir(dir).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(dir).unwrap().permissions();
            perms.set_mode(0o444);
            fs::set_permissions(dir, perms).unwrap();
        }

        let test_file = dir.join("test.cuentitos");
        let _runner = TestRunner::from_path(PathBuf::from("nonexistent"));
        let _test_case = create_test_case();

        let result = File::create(test_file);
        assert!(matches!(result, Err(_)));

        // Cleanup
        cleanup_dir(dir);
    }

    #[test]
    fn test_io_error_on_command_execution() {
        let runner = TestRunner::from_path(PathBuf::from("nonexistent_runtime"));
        let test_case = create_test_case();

        let result = runner.run(test_case);
        assert!(matches!(result, Err(TestError::IoError(_))));
    }

    #[test]
    fn test_cleanup_on_error() {
        let test_file = PathBuf::from("test.cuentitos");
        let _ = fs::remove_file(&test_file); // Clean up from any previous runs

        let runner = TestRunner::from_path(PathBuf::from("nonexistent_runtime"));
        let test_case = create_test_case();

        // Create the file first to ensure it exists
        let create_result = File::create(&test_file);
        dbg!(&create_result);
        create_result.unwrap();

        let exists_before = test_file.exists();
        dbg!(&exists_before);
        assert!(exists_before, "File should exist before running test");

        let run_result = runner.run(test_case);
        dbg!(&run_result);

        let exists_after = test_file.exists();
        dbg!(&exists_after);
        assert!(!exists_after, "File should not exist after running test");
    }

    #[test]
    fn test_utf8_error_handling() {
        // Create a mock runtime that outputs invalid UTF-8
        let mock_runtime = std::env::current_dir().unwrap().join("mock_runtime.sh");
        let invalid_utf8_file = std::env::current_dir().unwrap().join("invalid_utf8.bin");

        // Clean up from any previous failed runs
        let _ = fs::remove_file(&mock_runtime);
        let _ = fs::remove_file(&invalid_utf8_file);

        // Create a binary file with invalid UTF-8 bytes
        let write_result = fs::write(&invalid_utf8_file, &[0x80, 0x90, 0xA0]);
        dbg!(&write_result);
        write_result.unwrap();

        let exists = invalid_utf8_file.exists();
        dbg!(&exists);
        assert!(exists, "Invalid UTF-8 file should exist");

        // Create a script that outputs the invalid UTF-8 bytes directly
        let script = if cfg!(windows) {
            format!("@echo off\ntype {}", invalid_utf8_file.display())
        } else {
            // Echo the invalid UTF-8 bytes directly in the script
            String::from("#!/bin/sh\nprintf '\\x80\\x90\\xA0'")
        };
        dbg!(&script);

        let write_result = fs::write(&mock_runtime, script);
        dbg!(&write_result);
        write_result.unwrap();

        let exists = mock_runtime.exists();
        dbg!(&exists);
        assert!(exists, "Mock runtime script should exist");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&mock_runtime).unwrap().permissions();
            perms.set_mode(0o755);
            let chmod_result = fs::set_permissions(&mock_runtime, perms);
            dbg!(&chmod_result);
            chmod_result.unwrap();
        }

        let runner = TestRunner::from_path(mock_runtime.clone());
        let test_case = create_test_case();

        let result = runner.run(test_case);
        dbg!(&result);
        assert!(matches!(result, Err(TestError::Utf8Error(_))));

        // Cleanup
        let _ = fs::remove_file(&mock_runtime);
        let _ = fs::remove_file(&invalid_utf8_file);
    }

    #[test]
    fn test_error_display() {
        let io_error = TestError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test error"));
        // Create an invalid UTF-8 sequence for testing
        let utf8_error = TestError::Utf8Error(String::from_utf8(vec![0x80, 0x90, 0xA0]).unwrap_err());

        assert!(format!("{:?}", io_error).contains("IoError"));
        assert!(format!("{:?}", utf8_error).contains("Utf8Error"));
    }
}
