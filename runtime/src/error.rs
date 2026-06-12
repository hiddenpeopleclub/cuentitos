use cuentitos_common::{PathResolutionError, ValueKind};
use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during runtime execution
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// Section path could not be resolved
    SectionNotFound { path: String },
    /// Attempted to navigate above root level using ..
    NavigationAboveRoot,
    /// Invalid path syntax
    InvalidPath { message: String },
    /// Runtime is not currently running
    NotRunning,
    /// Attempted to read/write a variable that was never declared
    UndefinedVariable { name: String },
    /// Attempted to write a value whose variant doesn't match the variable's
    /// declared kind (e.g. assigning a string to an int).
    VariableTypeMismatch { name: String },
    /// An arithmetic expression evaluated at runtime divided by zero. The
    /// `line` is the source line that produced the error (used by the CLI
    /// to format `<file>:<line>: RUNTIME ERROR: Division by zero.`).
    DivisionByZero { file: Option<PathBuf>, line: usize },
    /// An arithmetic expression evaluated at runtime overflowed an `i64`.
    IntegerOverflow { file: Option<PathBuf>, line: usize },
    /// A float arithmetic expression evaluated at runtime overflowed the
    /// `f64` range to ±infinity. Parallel to
    /// [`IntegerOverflow`](Self::IntegerOverflow) but with a float-specific
    /// message.
    FloatOverflow { file: Option<PathBuf>, line: usize },
    /// A binary operator's operands had mismatched value kinds at
    /// runtime. Unreachable while `Value` has only the `Integer`
    /// variant — parser-time type inference catches every reachable
    /// mismatch on the call path that surfaces here — but the variant
    /// exists today so the runtime path doesn't `panic!` once a second
    /// `Value` variant lands.
    ///
    /// TODO: covered once Float/String land in `Value`. The fields
    /// match the upstream [`cuentitos_common::EvaluationError::TypeMismatch`]
    /// payload so the mapping is a direct field copy.
    EvaluationTypeMismatch {
        expected: ValueKind,
        found: ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::SectionNotFound { path } => {
                write!(f, "ERROR: Section not found: {}", path)
            }
            RuntimeError::NavigationAboveRoot => {
                write!(f, "ERROR: Cannot navigate above root level")
            }
            RuntimeError::InvalidPath { message } => {
                write!(f, "ERROR: Invalid goto command: {}", message)
            }
            RuntimeError::NotRunning => {
                write!(f, "ERROR: Runtime is not running")
            }
            RuntimeError::UndefinedVariable { name } => {
                write!(f, "ERROR: Undefined variable: '{}'", name)
            }
            RuntimeError::VariableTypeMismatch { name } => {
                write!(f, "ERROR: Type mismatch assigning to variable '{}'", name)
            }
            RuntimeError::DivisionByZero { file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("<script>");
                write!(f, "{}:{}: RUNTIME ERROR: Division by zero.", prefix, line)
            }
            RuntimeError::IntegerOverflow { file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("<script>");
                write!(f, "{}:{}: RUNTIME ERROR: Integer overflow.", prefix, line)
            }
            RuntimeError::FloatOverflow { file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("<script>");
                write!(f, "{}:{}: RUNTIME ERROR: Float overflow.", prefix, line)
            }
            RuntimeError::EvaluationTypeMismatch {
                expected,
                found,
                file,
                line,
            } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("<script>");
                write!(
                    f,
                    "{}:{}: RUNTIME ERROR: Type mismatch in expression: expected {}, found {}.",
                    prefix, line, expected, found
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<PathResolutionError> for RuntimeError {
    fn from(err: PathResolutionError) -> Self {
        match err {
            PathResolutionError::SectionNotFound { path } => RuntimeError::SectionNotFound { path },
            PathResolutionError::NavigationAboveRoot => RuntimeError::NavigationAboveRoot,
            PathResolutionError::InvalidPath { message } => RuntimeError::InvalidPath { message },
        }
    }
}
