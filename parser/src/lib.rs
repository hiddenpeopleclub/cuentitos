//! Parser for the cuentitos scripting language.
//!
//! The `cuentitos_parser` crate provides functionality to parse cuentitos scripts into a structured
//! database of blocks and strings. It handles:
//!
//! - Text blocks with proper indentation
//! - Section headers and hierarchies
//! - Parent-child relationships between blocks
//! - Error detection and reporting
//!
//! # Architecture
//!
//! The parser is built around several key components:
//!
//! - [`Parser`]: The main parser that coordinates the parsing process
//! - [`FeatureParser`]: A trait for specialized parsers that handle specific features
//! - [`ParserContext`]: Shared context between different parsers
//! - [`ParseError`]: Structured error types for various parsing failures
//!
//! # Example
//!
//! ```rust
//! use cuentitos_parser::parse;
//!
//! let script = "\
//! # Main Section
//!   This is some text
//!   ## Sub-section
//!     This is indented text
//! ";
//!
//! let database = parse(script).unwrap();
//! ```
//!
//! # Error Handling
//!
//! The parser provides detailed error messages for common issues:
//!
//! - Invalid indentation
//! - Empty section titles
//! - Orphaned sub-sections
//! - Duplicate section names
//!
//! Errors include file paths and line numbers when available, making it easy to
//! locate and fix issues in the source script.

use cuentitos_common::*;
use std::path::PathBuf;

pub mod parser;
pub mod parsers;

pub use parser::*;

#[cfg(test)]
mod tests;

/// Represents errors that can occur during parsing.
///
/// Each variant includes context about where and why the error occurred,
/// including file paths and line numbers when available.
#[derive(Debug, thiserror::Error, Clone)]
pub enum ParseError {
    /// A section header was found without a title
    #[error("{file}:{line}: ERROR: Section without title: found empty section title.")]
    EmptySectionTitle {
        /// The file where the error occurred
        file: PathBuf,
        /// The line number where the error occurred
        line: usize
    },

    /// A sub-section was found without a parent section
    #[error("{file}:{line}: ERROR: Invalid section hierarchy: found sub-section without parent section.")]
    OrphanedSubSection {
        /// The file where the error occurred
        file: PathBuf,
        /// The line number where the error occurred
        line: usize
    },

    /// Invalid number of spaces for indentation
    #[error("{file}:{line}: ERROR: Invalid indentation: found {spaces} spaces.")]
    InvalidIndentation {
        /// The file where the error occurred
        file: PathBuf,
        /// The line number where the error occurred
        line: usize,
        /// The number of spaces found
        spaces: usize,
    },

    /// A section name was used multiple times at the same level
    #[error("{file}:{line}: ERROR: Duplicate section name: '{name}' already exists at this level under '{parent}'. Previously defined at line {previous_line}.")]
    DuplicateSectionName {
        /// The file where the error occurred
        file: PathBuf,
        /// The line number where the error occurred
        line: usize,
        /// The duplicate section name
        name: String,
        /// The parent section name (or "<root>" for top-level sections)
        parent: String,
        /// The line number where this name was first used
        previous_line: usize,
    },
}

/// A collection of parsing errors.
///
/// This type is used to return multiple errors from a single parse operation,
/// allowing the parser to report all issues it finds rather than stopping at the first one.
#[derive(Debug, thiserror::Error)]
#[error("{}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n\n"))]
pub struct ParseErrors(pub Vec<ParseError>);

/// Parses a cuentitos script into a database.
///
/// This is a convenience function that creates a new parser and uses it to parse the script.
///
/// # Arguments
///
/// * `script` - The script text to parse
///
/// # Returns
///
/// * `Ok(Database)` - If parsing was successful
/// * `Err(ParseErrors)` - If any parsing errors occurred
pub fn parse(script: &str) -> Result<Database, ParseErrors> {
    let mut parser = Parser::new();
    parser.parse(script)
}
