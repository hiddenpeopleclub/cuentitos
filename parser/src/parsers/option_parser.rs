use super::{FeatureParser, ParserContext};
use crate::ParseError;

/// Parser for handling option lines (lines starting with `*`)
#[derive(Debug, Default)]
pub struct OptionParser;

/// The result of parsing an option line
#[derive(Debug)]
pub struct OptionParseResult {
    pub text: String,
}

impl OptionParser {
    pub fn new() -> Self {
        Self
    }

    /// Check if a line starts with `*` (after trimming whitespace)
    pub fn is_option_line(input: &str) -> bool {
        input.trim_start().starts_with('*')
    }
}

impl FeatureParser for OptionParser {
    type Output = OptionParseResult;
    type Error = ParseError;

    fn parse(
        &self,
        input: &str,
        _context: &mut ParserContext,
    ) -> Result<Self::Output, Self::Error> {
        let trimmed = input.trim_start();

        if !trimmed.starts_with('*') {
            return Err(ParseError::UnexpectedToken {
                file: _context.file_path.clone(),
                line: _context.current_line,
            });
        }

        // Extract option text after the `*` and trim it
        let text = trimmed[1..].trim().to_string();

        Ok(OptionParseResult { text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_option_line() {
        assert!(OptionParser::is_option_line("* Option text"));
        assert!(OptionParser::is_option_line("  * Option text"));
        assert!(OptionParser::is_option_line("*Option text"));
        assert!(!OptionParser::is_option_line("Not an option"));
        assert!(!OptionParser::is_option_line("// Comment"));
    }

    #[test]
    fn test_parse_option() {
        let parser = OptionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("* Go left", &mut context).unwrap();
        assert_eq!(result.text, "Go left");
    }

    #[test]
    fn test_parse_option_with_extra_spaces() {
        let parser = OptionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("*  Go left  ", &mut context).unwrap();
        assert_eq!(result.text, "Go left");
    }

    #[test]
    fn test_parse_option_with_indentation() {
        let parser = OptionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("  * Go right", &mut context).unwrap();
        assert_eq!(result.text, "Go right");
    }

    #[test]
    fn test_parse_non_option_fails() {
        let parser = OptionParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("Not an option", &mut context);
        assert!(result.is_err());
    }
}
