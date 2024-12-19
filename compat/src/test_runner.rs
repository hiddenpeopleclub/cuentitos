use crate::PathBuf;
use crate::TestCase;

#[derive(Debug, Clone)]
pub enum TestResult{
  Pass,
  Fail
}

pub struct TestRunner;

impl TestRunner {
  pub fn from_path(path: PathBuf) -> Self {
    TestRunner
  }

  pub fn run(&self, test_case: TestCase) -> TestResult {
    TestResult::Pass
  }
}

