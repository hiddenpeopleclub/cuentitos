use cuentitos_common::*;
use std::path::PathBuf;
use thiserror;

pub mod parser;
pub mod parsers;

pub use parser::*;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Empty section title at {file}:{line}")]
    EmptySectionTitle { file: PathBuf, line: usize },
    #[error("Invalid section hierarchy at {file}:{line}: found sub-section without parent section")]
    OrphanedSubSection { file: PathBuf, line: usize },
    #[error("Invalid indentation at {file}:{line}: found {spaces} spaces")]
    InvalidIndentation { file: PathBuf, line: usize, spaces: usize },
    #[error("Duplicate section name at {file}:{line}: '{name}' already exists at this level under '{parent}'. Previously defined at line {previous_line}")]
    DuplicateSectionName { file: PathBuf, line: usize, name: String, parent: String, previous_line: usize },
}

pub fn parse(script: &str) -> Result<Database, ParseError> {
    let mut parser = Parser::new();
    parser.parse(script)
}
