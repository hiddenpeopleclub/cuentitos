use super::{FeatureParser, ParserContext};
use crate::ParseError;

/// Parser for handling go-to-section-and-back commands (e.g., <-> Section Name, <-> .. \ Sibling)
#[derive(Debug, Default)]
pub struct GoToSectionAndBackParser;

/// The result of parsing a go-to-section-and-back command
#[derive(Debug)]
pub struct GoToSectionAndBackParseResult {
    pub path: String,
}

impl GoToSectionAndBackParser {
    pub fn new() -> Self {
        Self
    }

    /// Check if a line is a go-to-section-and-back command (starts with <->)
    pub fn is_go_to_section_and_back(input: &str) -> bool {
        input.trim_start().starts_with("<->")
    }
}

impl FeatureParser for GoToSectionAndBackParser {
    type Output = Option<GoToSectionAndBackParseResult>;
    type Error = ParseError;

    fn parse(&self, input: &str, context: &mut ParserContext) -> Result<Self::Output, Self::Error> {
        let trimmed = input.trim_start();

        // Check if this is a go-to-section-and-back command
        if !trimmed.starts_with("<->") {
            return Ok(None);
        }

        // Must have at least one space after <->
        if !trimmed.starts_with("<-> ") {
            return Err(ParseError::InvalidGoToSection {
                message: "Expected section name after '<->'".to_string(),
                file: context.file_path.clone(),
                line: context.current_line,
            });
        }

        // Get the path after "<-> " (including any extra spaces)
        let path = &trimmed[4..]; // Skip "<-> "

        // Check if path is empty or whitespace only
        if path.trim().is_empty() {
            return Err(ParseError::InvalidGoToSection {
                message: "Expected section name after '<->'".to_string(),
                file: context.file_path.clone(),
                line: context.current_line,
            });
        }

        // Validate spacing in path (no double spaces, proper spacing around \)
        if let Err(msg) = Self::validate_path_spacing(path) {
            return Err(ParseError::InvalidGoToSection {
                message: msg,
                file: context.file_path.clone(),
                line: context.current_line,
            });
        }

        Ok(Some(GoToSectionAndBackParseResult {
            path: path.to_string(),
        }))
    }
}

impl GoToSectionAndBackParser {
    /// Validate spacing rules in the path
    fn validate_path_spacing(path: &str) -> Result<(), String> {
        // Check for trailing backslash
        if path.trim_end().ends_with('\\') {
            return Err("Expected section name after '<->'".to_string());
        }

        // If path contains \, validate spacing around it
        // The pattern should be " \ " (space-backslash-space)
        if path.contains('\\') {
            // Split by " \ " and check that we get proper parts
            let parts: Vec<&str> = path.split(" \\ ").collect();

            // If splitting by correct pattern doesn't match the number of backslashes + 1,
            // then spacing is wrong
            let backslash_count = path.matches('\\').count();
            if parts.len() != backslash_count + 1 {
                return Err("Expected section name after '<->'".to_string());
            }

            // Check that no part is empty (would indicate " \ \ " or "\ " or " \" patterns)
            for part in &parts {
                if part.trim().is_empty() {
                    return Err("Expected section name after '<->'".to_string());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_call_and_return() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("<-> Section B", &mut context).unwrap().unwrap();
        assert_eq!(result.path, "Section B");
    }

    #[test]
    fn test_parse_absolute_path() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser
            .parse("<-> Root \\ Child", &mut context)
            .unwrap()
            .unwrap();
        assert_eq!(result.path, "Root \\ Child");
    }

    #[test]
    fn test_parse_parent_path() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("<-> ..", &mut context).unwrap().unwrap();
        assert_eq!(result.path, "..");
    }

    #[test]
    fn test_parse_current_section() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("<-> .", &mut context).unwrap().unwrap();
        assert_eq!(result.path, ".");
    }

    #[test]
    fn test_parse_combined_path() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser
            .parse("<-> .. \\ Sibling", &mut context)
            .unwrap()
            .unwrap();
        assert_eq!(result.path, ".. \\ Sibling");
    }

    #[test]
    fn test_parse_non_call_and_return() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("Just regular text", &mut context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_regular_goto() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("-> Section B", &mut context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_no_space_after_arrow() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("<->Section B", &mut context);
        assert!(matches!(result, Err(ParseError::InvalidGoToSection { .. })));
    }

    #[test]
    fn test_parse_empty_reference() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("<->", &mut context);
        assert!(matches!(result, Err(ParseError::InvalidGoToSection { .. })));
    }

    #[test]
    fn test_parse_trailing_backslash() {
        let parser = GoToSectionAndBackParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("<-> Section \\", &mut context);
        assert!(matches!(result, Err(ParseError::InvalidGoToSection { .. })));
    }

    #[test]
    fn test_is_go_to_section_and_back() {
        assert!(GoToSectionAndBackParser::is_go_to_section_and_back("<-> Section"));
        assert!(GoToSectionAndBackParser::is_go_to_section_and_back("  <-> Section"));
        assert!(!GoToSectionAndBackParser::is_go_to_section_and_back("Regular text"));
        assert!(!GoToSectionAndBackParser::is_go_to_section_and_back("# Section"));
        assert!(!GoToSectionAndBackParser::is_go_to_section_and_back("-> Section"));
    }
}
