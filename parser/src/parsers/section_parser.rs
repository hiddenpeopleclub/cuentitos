use super::{FeatureParser, ParserContext};
use crate::ParseError;
use std::path::PathBuf;

/// Parser for handling section headers (lines starting with #).
///
/// This parser is responsible for:
/// - Detecting section headers (lines starting with #)
/// - Determining section levels based on # count
/// - Extracting section titles
/// - Validating section header format
#[derive(Debug, Default)]
pub struct SectionParser;

/// The result of parsing a section header.
///
/// Contains the extracted title and level information from a section header.
/// The level is zero-based (# = 0, ## = 1, etc.).
#[derive(Debug)]
pub struct SectionParseResult {
    /// The title of the section, with leading/trailing whitespace removed
    pub title: String,
    /// The zero-based level of the section (# = 0, ## = 1, etc.)
    pub level: usize,
}

impl SectionParser {
    /// Creates a new instance of the section parser.
    pub fn new() -> Self {
        Self
    }

    /// Counts the number of consecutive '#' characters at the start of a line.
    ///
    /// # Arguments
    ///
    /// * `input` - The line of text to analyze
    ///
    /// # Returns
    ///
    /// The number of consecutive '#' characters at the start of the line
    fn count_section_level(&self, input: &str) -> usize {
        input.chars().take_while(|c| *c == '#').count()
    }
}

impl FeatureParser for SectionParser {
    type Output = Option<SectionParseResult>;
    type Error = ParseError;

    /// Parses a line of text to determine if it's a section header and extract its information.
    ///
    /// # Arguments
    ///
    /// * `input` - The line of text to parse
    /// * `context` - The current parsing context
    ///
    /// # Returns
    ///
    /// * `Ok(Some(SectionParseResult))` - If the line is a valid section header
    /// * `Ok(None)` - If the line is not a section header
    /// * `Err(ParseError)` - If the line is an invalid section header (e.g., empty title)
    fn parse(&self, input: &str, context: &mut ParserContext) -> Result<Self::Output, Self::Error> {
        // Check if line starts with #
        if !input.starts_with('#') {
            return Ok(None);
        }

        // Count the number of # symbols
        let level = self.count_section_level(input);

        // Extract the title (skip the # symbols and any whitespace)
        let title = input[level..].trim().to_string();
        if title.is_empty() {
            return Err(ParseError::EmptySectionTitle {
                file: context
                    .file_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("<unknown>")),
                line: context.current_line,
            });
        }

        Ok(Some(SectionParseResult {
            title,
            level: level - 1,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_section() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("# Section Title", &mut context).unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.title, "Section Title");
        assert_eq!(result.level, 0);
    }

    #[test]
    fn test_parse_subsection() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("### Deep Section", &mut context).unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.title, "Deep Section");
        assert_eq!(result.level, 2);
    }

    #[test]
    fn test_parse_non_section() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("Regular text", &mut context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_section() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("#", &mut context);
        assert!(result.is_err());
    }
}
