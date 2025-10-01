use super::{FeatureParser, ParserContext};
use crate::ParseError;

/// Parser for handling section headers (e.g., # Section Name, ## Subsection)
#[derive(Debug, Default)]
pub struct SectionParser;

/// The result of parsing a section header
#[derive(Debug)]
pub struct SectionParseResult {
    pub id: String,
    pub display_name: String,
    pub hash_count: usize,
}

impl SectionParser {
    pub fn new() -> Self {
        Self
    }

    /// Check if a line is a section header (starts with #)
    pub fn is_section(input: &str) -> bool {
        input.trim_start().starts_with('#')
    }
}

impl FeatureParser for SectionParser {
    type Output = Option<SectionParseResult>;
    type Error = ParseError;

    fn parse(&self, input: &str, _context: &mut ParserContext) -> Result<Self::Output, Self::Error> {
        let trimmed = input.trim_start();

        // Check if this is a section header
        if !trimmed.starts_with('#') {
            return Ok(None);
        }

        // Count the number of '#' symbols
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

        // Get the rest of the text after the '#' symbols
        let rest = trimmed[hash_count..].trim();

        // Check for empty section title
        if rest.is_empty() {
            return Err(ParseError::SectionWithoutTitle {
                file: _context.file_path.clone(),
                line: _context.current_line,
            });
        }

        // Parse the format - can be either:
        // "Display Name" (without ID - use display name as ID)
        let (section_id, display_name) = if rest.contains(':') {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            if parts.len() == 2 {
                (parts[0].trim().to_string(), parts[1].trim().to_string())
            } else {
                (rest.to_string(), rest.to_string())
            }
        } else {
            // Use the display name as the ID when no ID is provided
            (rest.to_string(), rest.to_string())
        };

        Ok(Some(SectionParseResult {
            id: section_id,
            display_name,
            hash_count,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_section() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("# First Section", &mut context).unwrap().unwrap();
        assert_eq!(result.id, "First Section");
        assert_eq!(result.display_name, "First Section");
        assert_eq!(result.hash_count, 1);
    }

    #[test]
    fn test_parse_section_with_id() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("# section_1: First Section", &mut context).unwrap().unwrap();
        assert_eq!(result.id, "section_1");
        assert_eq!(result.display_name, "First Section");
        assert_eq!(result.hash_count, 1);
    }

    #[test]
    fn test_parse_subsection() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("## Subsection", &mut context).unwrap().unwrap();
        assert_eq!(result.hash_count, 2);
    }

    #[test]
    fn test_parse_non_section() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("Just regular text", &mut context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_section_title() {
        let parser = SectionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("#", &mut context);
        match result {
            Err(ParseError::SectionWithoutTitle { .. }) => {}
            _ => panic!("Expected SectionWithoutTitle error"),
        }
    }

    #[test]
    fn test_is_section() {
        assert!(SectionParser::is_section("# Section"));
        assert!(SectionParser::is_section("  ## Indented Section"));
        assert!(!SectionParser::is_section("Regular text"));
        assert!(!SectionParser::is_section("No hash here"));
    }
}
