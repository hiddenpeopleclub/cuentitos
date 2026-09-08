use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

use crate::PathBuf;
use crate::TestCase;
use cuentitos_common::test_case::TranslationFile;

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
        let _ = std::fs::remove_dir_all(self.dir.join("locales"));
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

        let locales_dir = temp_path.parent().unwrap().join("locales");
        if !test_case.translations.is_empty() {
            std::fs::create_dir_all(&locales_dir).unwrap();
            for translation in &test_case.translations {
                let path = locales_dir.join(format!("{}.csv", translation.locale));
                let mut file = File::create(&path).unwrap();
                writeln!(file, "{}", translation.contents).unwrap();
            }
        }

        let input_commands = test_case.input.split("\n").collect::<Vec<&str>>().join(",");

        // Run the runtime with the script file and the input from the test case.
        // Concatenate stderr after stdout so runtime errors (printed to stderr
        // by the CLI) appear in the captured transcript alongside narrative
        // output.
        let result = match Command::new(&self.runtime_path)
            .arg("run")
            .arg(&temp_path)
            .arg(&input_commands)
            .output()
        {
            Ok(result) => {
                let mut output = String::from_utf8(result.stdout.clone()).unwrap_or_default();
                // Append only error-bearing stderr lines (e.g. RUNTIME ERROR)
                // to the captured transcript. Informational stderr lines
                // (lines starting with `Warning:`) aren't part of test
                // expectations and would otherwise leak into every
                // transcript that doesn't reach END.
                let stderr = String::from_utf8(result.stderr.clone()).unwrap_or_default();
                for line in stderr.lines() {
                    if line.trim_start().starts_with("Warning:") {
                        continue;
                    }
                    output.push_str(line);
                    output.push('\n');
                }
                let output_trimmed = output.trim_end_matches(&['\r', '\n'][..]);
                let expected_trimmed = test_case.result.trim_end_matches(&['\r', '\n'][..]);

                if expected_trimmed != output_trimmed {
                    TestResult::Fail {
                        expected: Some(test_case.result),
                        actual: output,
                    }
                } else {
                    match compare_translations(&locales_dir, &test_case.expected_translations) {
                        Some(failure) => failure,
                        None => TestResult::Pass,
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

/// Compares each expected translation file against what the run left under
/// `locales/`. Returns the first mismatch, or `None` when every file matches.
fn compare_translations(
    locales_dir: &std::path::Path,
    expected_translations: &[TranslationFile],
) -> Option<TestResult> {
    for expected in expected_translations {
        let path = locales_dir.join(format!("{}.csv", expected.locale));
        let actual = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(_) => {
                return Some(TestResult::Fail {
                    expected: Some(label(expected.locale.as_str(), &expected.contents)),
                    actual: format!("locales/{}.csv was not written", expected.locale),
                })
            }
        };

        let trimmed = actual.trim_end_matches(&['\r', '\n'][..]);
        if trimmed != expected.contents {
            return Some(TestResult::Fail {
                expected: Some(label(expected.locale.as_str(), &expected.contents)),
                actual: label(expected.locale.as_str(), trimmed),
            });
        }
    }

    None
}

fn label(locale: &str, contents: &str) -> String {
    format!("locales/{}.csv:\n{}", locale, contents)
}
