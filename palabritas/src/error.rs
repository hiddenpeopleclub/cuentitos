use std::{error::Error, fmt::Display, path::PathBuf};

use cuentitos_common::SectionKey;

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
  SectionDoesntExist {
    info: ErrorInfo,
    section: SectionKey,
  },
  VariableDoesntExist {
    info: ErrorInfo,
    variable: String,
  },
  InvalidVariableValue {
    info: ErrorInfo,
    variable: String,
    value: String,
    variable_type: String,
  },
  InvalidVariableOperator {
    info: ErrorInfo,
    variable: String,
    operator: String,
    variable_type: String,
  },
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
      PalabritasError::VariableDoesntExist { info, variable } => {
        write!(f, "Variable {} doesn't exist.\n{}", variable, info)
      }
      PalabritasError::SectionDoesntExist { info, section } => {
        write!(f, "Section {} doesn't exist.\n{}", section, info)
      }
      PalabritasError::InvalidVariableValue {
        info,
        variable,
        value,
        variable_type,
      } => {
        write!(
          f,
          "Invalid value for variable {}. Expected {}, but found {}\n{}",
          variable, value, variable_type, info
        )
      }
      PalabritasError::InvalidVariableOperator {
        info,
        variable,
        operator,
        variable_type,
      } => {
        write!(
          f,
          "Invalid operator for variable {}. Operator {} can't be applied to {}\n{}",
          variable, operator, variable_type, info
        )
      }
    }
  }
}
