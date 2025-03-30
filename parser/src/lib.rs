use cuentitos_common::*;
use std::path::PathBuf;
use thiserror;

pub mod parser;
pub mod parsers;

pub use parser::*;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error, Clone)]
pub enum ParseError {
    #[error("{file}:{line}: ERROR: Section without title: found empty section title.")]
    EmptySectionTitle { file: PathBuf, line: usize },
    #[error("{file}:{line}: ERROR: Invalid section hierarchy: found sub-section without parent section.")]
    OrphanedSubSection { file: PathBuf, line: usize },
    #[error("{file}:{line}: ERROR: Invalid indentation: found {spaces} spaces.")]
    InvalidIndentation { file: PathBuf, line: usize, spaces: usize },
    #[error("{file}:{line}: ERROR: Duplicate section name: '{name}' already exists at this level under '{parent}'. Previously defined at line {previous_line}.")]
    DuplicateSectionName { file: PathBuf, line: usize, name: String, parent: String, previous_line: usize },
}

#[derive(Debug, thiserror::Error)]
#[error("{}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n\n"))]
pub struct ParseErrors(pub Vec<ParseError>);

pub fn parse(script: &str) -> Result<Database, ParseErrors> {
    let mut parser = Parser::new();
    parser.parse(script)
}
