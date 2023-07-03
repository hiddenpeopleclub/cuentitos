use std::{error::Error, fmt::Display, path::PathBuf};

use cuentitos_common::SectionKey;

use crate::parser::Rule;
#[derive(Debug, PartialEq, Eq)]
pub enum PalabritasError {
  FileIsEmpty,
  ParseError {
    file: String,
    info: ErrorInfo,
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
  pub col: usize,
  pub string: String,
}

impl Display for ErrorInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}:{}  {}", self.line, self.col, self.string)
  }
}
impl Error for PalabritasError {}
impl Display for PalabritasError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      PalabritasError::FileIsEmpty => {
        write!(f, "File provided is empty.")
      }
      PalabritasError::ParseError { file, info } => {
        write!(f, "{}:{}", file, info)
      }
      PalabritasError::BucketSumIsNot1(info) => {
        write!(
          f,
          "{}\n  The sum of the probabilities in a bucket must be 100%.",
          info
        )
      }
      PalabritasError::BucketHasFrequenciesAndChances(info) => {
        write!(
          f,
          "{}\n  Buckets can't have frequency notations and percentage notations at the same time.",
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
          "{}\n  Expected {:?} but found {:?}.",
          info, expected_rule, rule_found
        )
      }
      PalabritasError::BucketMissingProbability(info) => {
        write!(f, "{}\n  Missing probability for bucket element.\n", info)
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
      PalabritasError::DivisionByZero(info) => {
        write!(f, "{}\n  Can't divide by zero.", info)
      }
      PalabritasError::VariableDoesntExist { info, variable } => {
        write!(f, "{}\n  Variable '{}' doesn't exist.", info, variable)
      }
      PalabritasError::SectionDoesntExist { info, section } => {
        write!(f, "{}\n  Section '{}' doesn't exist.", info, section)
      }
      PalabritasError::InvalidVariableValue {
        info,
        variable,
        value,
        variable_type,
      } => {
        write!(
          f,
          "{}\n  Invalid value for variable '{}'. Expected {}, but found '{}'",
          info, variable, variable_type, value,
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
          "{}\n  Invalid operator for variable '{}'. Operator '{}' can't be applied to {}",
          info, variable, operator, variable_type
        )
      }
    }
  }
}
