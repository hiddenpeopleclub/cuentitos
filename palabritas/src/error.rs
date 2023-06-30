use std::{error::Error, fmt::Display, path::PathBuf};

use crate::parser::Rule;
#[derive(Debug, PartialEq, Eq)]
pub enum PalabritasError {
  FileIsEmpty,
  ParseError {
    file: String,
    line: usize,
    col: usize,
    reason: String,
  },
  PathIsNotAFile(PathBuf),
  PathDoesntExist(PathBuf),
  CantReadFile {
    path: PathBuf,
    message: String,
  },
  BucketSumIsNot1(ErrorInfo),
  BucketHasFrequenciesAndChances(ErrorInfo),
  BucketMissingProbability(ErrorInfo),
  UnexpectedRule {
    info: ErrorInfo,
    expected_rule: Rule,
    rule_found: Rule,
  },
  DivisionByZero(ErrorInfo),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ErrorInfo {
  pub line: usize,
  pub string: String,
}

impl Display for ErrorInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Line {}: {}", self.line, self.string)
  }
}
impl Error for PalabritasError {}
impl Display for PalabritasError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      PalabritasError::FileIsEmpty => {
        write!(f, "File provided is empty.")
      }
      PalabritasError::ParseError {
        file,
        line,
        col,
        reason,
      } => {
        write!(f, "{}:{}:{}\n  {}", file, line, col, reason)
      }
      PalabritasError::BucketSumIsNot1(info) => {
        write!(
          f,
          "The sum of the probabilities in a bucket must be 100%.\n{}",
          info
        )
      }
      PalabritasError::BucketHasFrequenciesAndChances(info) => {
        write!(
          f,
          "Buckets can't have frequency notations and percentage notations at the same time.\n{}",
          info
        )
      }
      PalabritasError::UnexpectedRule {
        info,
        expected_rule,
        rule_found,
      } => {
        write!(
          f,
          "Expected {:?} but found {:?}.\n{}",
          expected_rule, rule_found, info
        )
      }
      PalabritasError::BucketMissingProbability(info) => {
        write!(f, "Missing probability for bucket element.\n{}", info)
      }
      PalabritasError::CantReadFile { path, message } => {
        write!(f, "Can't read file {:?}\n{}", path, message)
      }
      PalabritasError::PathIsNotAFile(path) => {
        write!(f, "{:?} is not a file", path)
      }
      PalabritasError::PathDoesntExist(path) => {
        write!(f, "Path provided doesn't exist: {:?}", path)
      }
      PalabritasError::DivisionByZero(path) => {
        write!(f, "Can't divide by zero: {:?}", path)
      }
    }
  }
}
