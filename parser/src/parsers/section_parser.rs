use super::{FeatureParser, ParserContext};
use crate::ParseError;
use std::path::PathBuf;

/// Parser for handling section headers (lines starting with #)
#[derive(Debug, Default)]
pub struct SectionParser;

/// The result of parsing a section header
#[derive(Debug)]
pub struct SectionParseResult {
    pub title: String,
    pub level: usize,
}

impl SectionParser {
    pub fn new() -> Self {
        Self
    }

    fn count_section_level(&self, input: &str) -> usize {
        input.chars().take_while(|c| *c == '#').count()
    }
}

impl FeatureParser for SectionParser {
    type Output = Option<SectionParseResult>;
    type Error = ParseError;

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
