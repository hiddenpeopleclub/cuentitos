use cuentitos_common::PathResolutionError;
use std::fmt;

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
    /// Variable requirement failed
    RequirementFailed { message: String },
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
            RuntimeError::RequirementFailed { message } => {
                write!(f, "ERROR: Requirement failed: {}", message)
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
