use cuentitos_common::{BlockId, Section};
use std::fmt::Debug;
use std::{
  error::Error,
  fmt::Display,
  num::{ParseFloatError, ParseIntError},
  str::ParseBoolError,
};
type VariableName = String;

#[derive(PartialEq, Eq)]
pub enum RuntimeError {
  MissingLocale(String),
  RewindWithNoHistory(),
  RewindWithToInvalidIndex {
    index: usize,
    current_index: usize,
  },
  InvalidBlockId(BlockId),
  WaitingForChoice(Vec<String>),
  StoryFinished,
  SectionDoesntExist(Section),
  UnexpectedBlock {
    expected_block: String,
    block_found: String,
  },
  EmptyStack,
  EmptyDatabase,
  NoChoices,
  InvalidChoice {
    total_choices: usize,
    choice_picked: usize,
  },
  UnsupportedVariableType {
    type_found: String,
  },
  VariableDoesntExist(VariableName),
  ParseIntError(ParseIntError),
  ParseFloatError(ParseFloatError),
  ParseBoolError(ParseBoolError),
  UnknownParsingError,
  FrequencyModifierWithProbability,
  FrequencyOutOfBucket,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorInfo {
  pub line: usize,
  pub col: usize,
  pub string: String,
}

impl Display for ErrorInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}:{}\n  {}", self.line, self.col, self.string)
  }
}
impl Error for RuntimeError {}
impl Debug for RuntimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}
impl Display for RuntimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      RuntimeError::InvalidBlockId(id) => {
        write!(f, "Attempted to access invalid block id {}.", id)
      }
      RuntimeError::WaitingForChoice(choices) => {
        let mut choices_str = String::default();
        for choice in choices {
          choices_str += &format!("\n  -{}", choice)
        }
        write!(
          f,
          "Can't progress story without making a choice.{}",
          choices_str
        )
      }
      RuntimeError::StoryFinished => {
        write!(f, "Story finished.")
      }
      RuntimeError::SectionDoesntExist(section_name) => {
        write!(f, "Section `{}` doesnt exist.", section_name)
      }
      RuntimeError::UnexpectedBlock {
        expected_block,
        block_found,
      } => {
        write!(
          f,
          "Expected `{}` but found `{}`",
          expected_block, block_found
        )
      }
      RuntimeError::EmptyDatabase => {
        write!(f, "Story is empty.")
      }
      RuntimeError::NoChoices => {
        write!(f, "Can't make a choice because there are no options.")
      }
      RuntimeError::InvalidChoice {
        total_choices,
        choice_picked,
      } => {
        write!(
          f,
          "Can't pick choice `{}` because there's only `{}` options.",
          choice_picked, total_choices
        )
      }
      RuntimeError::UnsupportedVariableType { type_found } => {
        write!(f, "`{}` is not a supported type for variables.\nSupported types:\n  -Integer\n  -Float\n  -Bool\n  -String\n  -Enum", type_found)
      }
      RuntimeError::VariableDoesntExist(variable) => {
        write!(f, "Variable `{}` does not exist.", variable)
      }
      RuntimeError::ParseIntError(e) => {
        write!(f, "{}", e)
      }
      RuntimeError::ParseFloatError(e) => {
        write!(f, "{}", e)
      }
      RuntimeError::ParseBoolError(e) => {
        write!(f, "{}", e)
      }
      RuntimeError::UnknownParsingError => {
        write!(f, "Unknown parsing error.")
      }
      RuntimeError::EmptyStack => {
        write!(f, "The story has not been started.")
      }
      RuntimeError::FrequencyModifierWithProbability => {
        write!(f, "Can't apply a frequency modifier to a probability.")
      }
      RuntimeError::FrequencyOutOfBucket => {
        write!(f, "Frequencies are only allowed inside of buckets.")
      }
      RuntimeError::RewindWithNoHistory() => {
        write!(f, "Can't rewind when the history is empty.")
      }
      RuntimeError::MissingLocale(s) => {
        write!(f, "Missing locale: {}", s)
      }
      RuntimeError::RewindWithToInvalidIndex {
        index,
        current_index,
      } => {
        write!(
          f,
          "Can't rewind to {} because the current index is {}.",
          index, current_index
        )
      }
    }
  }
}
