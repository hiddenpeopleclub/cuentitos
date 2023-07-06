use std::{error::Error, fmt::Display, path::PathBuf};

use cuentitos_common::{Section, VariableKind};

use crate::parser::Rule;
#[derive(Debug, PartialEq, Eq)]
pub enum PalabritasError {
  FileIsEmpty,
  ParseError {
    info: ErrorInfo,
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
    section: Section,
  },
  VariableDoesntExist {
    info: ErrorInfo,
    variable: String,
  },
  InvalidVariableValue {
    info: Box<ErrorInfo>,
    variable: String,
    value: String,
    variable_type: VariableKind,
  },
  InvalidVariableOperator {
    info: Box<ErrorInfo>,
    variable: String,
    operator: String,
    variable_type: VariableKind,
  },
  FrequencyOutOfBucket(ErrorInfo),
  FrequencyModifierWithoutFrequencyChance(ErrorInfo),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorInfo {
  pub line: usize,
  pub col: usize,
  pub string: String,
  pub file: String,
}

impl Display for ErrorInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "{}:{}:{}  {}",
      self.file, self.line, self.col, self.string
    )
  }
}
impl Error for PalabritasError {}
impl Display for PalabritasError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      PalabritasError::FileIsEmpty => {
        write!(f, "File provided is empty.")
      }
      PalabritasError::ParseError { info, reason } => {
        write!(f, "{}\n{}", info, reason)
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
      PalabritasError::FrequencyOutOfBucket(info) => {
        write!(
          f,
          "{}\n  Frequency notation is only allowed in buckets.",
          info
        )
      }
      PalabritasError::FrequencyModifierWithoutFrequencyChance(info) => {
        write!(
          f,
          "{}\n  Frequency modifiers are only allowed for blocks with probabilistic chance and frequency notation.",
          info
        )
      }
    }
  }
}
