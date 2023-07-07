use crate::parser::Rule;
use cuentitos_common::{Script, Section, SectionName, VariableKind};
use std::{error::Error, fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum PalabritasError {
  FileIsEmpty,
  ParseError {
    script: Script,
    reason: String,
  },
  PathIsNotAFile(PathBuf),
  PathDoesntExist(PathBuf),
  CantReadFile {
    path: PathBuf,
    message: String,
  },
  BucketSumIsNot1(Script, String),
  BucketHasFrequenciesAndChances(Script, String),
  BucketMissingProbability(Script, String),
  UnexpectedRule {
    script: Script,
    expected_rule: Rule,
    rule_found: Rule,
  },
  DivisionByZero(Script, String),
  SectionDoesntExist {
    script: Script,
    section: Section,
  },
  DuplicatedSection {
    first_appearance: Box<Script>,
    second_appearance: Box<Script>,
    section_name: Section,
  },
  SubsectioNamedAfterSection {
    subsection_script: Box<Script>,
    section_script: Box<Script>,
    section_name: SectionName,
  },
  VariableDoesntExist {
    script: Script,
    string: String,
    variable: String,
  },
  InvalidVariableValue {
    script: Box<Script>,
    string: String,
    variable: String,
    value: String,
    variable_type: VariableKind,
  },
  InvalidVariableOperator {
    script: Box<Script>,
    string: String,
    variable: String,
    operator: String,
    variable_type: VariableKind,
  },
  FrequencyOutOfBucket(Script, String),
  FrequencyModifierWithoutFrequencyChance(Script, String),
}

impl Error for PalabritasError {}
impl Display for PalabritasError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      PalabritasError::FileIsEmpty => {
        write!(f, "File provided is empty.")
      }
      PalabritasError::ParseError { script, reason } => {
        write!(f, "{}\n{}", script, reason)
      }
      PalabritasError::BucketSumIsNot1(script, string) => {
        write!(
          f,
          "{}  {}\n  The sum of the probabilities in a bucket must be 100%.",
          script, string
        )
      }
      PalabritasError::BucketHasFrequenciesAndChances(script, string) => {
        write!(
          f,
          "{}  {}\n  Buckets can't have frequency notations and percentage notations at the same time.",
          script,
          string
        )
      }
      PalabritasError::UnexpectedRule {
        script,
        expected_rule,
        rule_found,
      } => {
        write!(
          f,
          "{}\n  Expected {:?} but found {:?}.",
          script, expected_rule, rule_found
        )
      }
      PalabritasError::BucketMissingProbability(script, string) => {
        write!(
          f,
          "{}  {}\n  Missing probability for bucket element.\n",
          script, string
        )
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
      PalabritasError::DivisionByZero(script, string) => {
        write!(f, "{}  {}\n  Can't divide by zero.", script, string)
      }
      PalabritasError::VariableDoesntExist {
        script,
        string,
        variable,
      } => {
        write!(
          f,
          "{}  {}\n  Variable '{}' doesn't exist.",
          script, string, variable
        )
      }
      PalabritasError::SectionDoesntExist { script, section } => {
        write!(f, "{}\n  Section '{}' doesn't exist.", script, section)
      }
      PalabritasError::InvalidVariableValue {
        script,
        string,
        variable,
        value,
        variable_type,
      } => {
        write!(
          f,
          "{}  {}\n  Invalid value for variable '{}'. Expected {}, but found '{}'",
          script, string, variable, variable_type, value,
        )
      }
      PalabritasError::InvalidVariableOperator {
        script,
        string,
        variable,
        operator,
        variable_type,
      } => {
        write!(
          f,
          "{}  {}\n  Invalid operator for variable '{}'. Operator '{}' can't be applied to {}",
          script, string, variable, operator, variable_type
        )
      }
      PalabritasError::FrequencyOutOfBucket(script, string) => {
        write!(
          f,
          "{}  {}\n  Frequency notation is only allowed in buckets.",
          script, string
        )
      }
      PalabritasError::FrequencyModifierWithoutFrequencyChance(script, string) => {
        write!(
          f,
          "{}  {}\n  Frequency modifiers are only allowed for blocks with probabilistic chance and frequency notation.",
          script,
          string
        )
      }
      PalabritasError::DuplicatedSection {
        first_appearance,
        second_appearance,
        section_name,
      } => {
        write!(
          f,
          "{}\n  Tried to define a new section named `{}` but it was already defined in {}
            ",
          second_appearance, section_name, first_appearance
        )
      }
      PalabritasError::SubsectioNamedAfterSection {
        subsection_script,
        section_script,
        section_name,
      } => {
        write!(
            f,
            "{}\n  Tried to define a subsection named `{}` but there is a section with the same name defined in {}
              ",
              subsection_script, section_name, section_script
          )
      }
    }
  }
}
