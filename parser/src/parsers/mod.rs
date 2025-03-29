use cuentitos_common::*;

pub mod line_parser;
pub mod section_parser;

/// Represents the shared context between different parsers
#[derive(Debug)]
pub struct ParserContext {
    /// The current line being processed
    pub current_line: usize,
    /// The current indentation level
    pub current_level: usize,
    /// The file path being processed, if any
    pub file_path: Option<std::path::PathBuf>,
    /// The database being built
    pub database: Database,
}

impl Default for ParserContext {
    fn default() -> Self {
        Self {
            current_line: 1,
            current_level: 0,
            file_path: None,
            database: Database::new(),
        }
    }
}

impl ParserContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path: Some(file_path),
            ..Self::default()
        }
    }
}

/// The core trait that all feature parsers must implement
pub trait FeatureParser {
    /// The type of output this parser produces
    type Output;
    /// The type of error this parser can produce
    type Error;

    /// Parse the input string using the given context
    ///
    /// # Arguments
    /// * `input` - The input string to parse
    /// * `context` - The shared parser context
    fn parse(&self, input: &str, context: &mut ParserContext) -> Result<Self::Output, Self::Error>;
}
