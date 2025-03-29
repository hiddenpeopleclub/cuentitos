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
    #[error("{file}:{line}: ERROR: Section without title: found empty section title.")]
    EmptySectionTitle { file: PathBuf, line: usize },
    #[error("{file}:{line}: ERROR: Invalid section hierarchy: found sub-section without parent section.")]
    OrphanedSubSection { file: PathBuf, line: usize },
    #[error("{file}:{line}: ERROR: Invalid indentation: found {spaces} spaces.")]
    InvalidIndentation { file: PathBuf, line: usize, spaces: usize },
    #[error("Duplicate section name at {file}:{line}: '{name}' already exists at this level under '{parent}'. Previously defined at line {previous_line}")]
    DuplicateSectionName { file: PathBuf, line: usize, name: String, parent: String, previous_line: usize },
}

pub fn parse(script: &str) -> Result<Database, ParseError> {
    let mut parser = Parser::new();
    parser.parse(script)
}
