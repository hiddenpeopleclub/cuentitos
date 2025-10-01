use super::{FeatureParser, ParserContext};
use crate::ParseError;

/// Parser for handling basic text lines
#[derive(Debug, Default)]
pub struct LineParser;

/// The result of parsing a line
#[derive(Debug)]
pub struct LineParseResult {
    pub string: String,
}

impl LineParser {
    pub fn new() -> Self {
        Self
    }
}

impl FeatureParser for LineParser {
    type Output = LineParseResult;
    type Error = ParseError;

    fn parse(&self, input: &str, _context: &mut ParserContext) -> Result<Self::Output, Self::Error> {
        Ok(LineParseResult {
            string: input.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let parser = LineParser::new();
        let mut context = ParserContext::new();

        let result = parser.parse("Hello, world!", &mut context).unwrap();
        assert_eq!(result.string, "Hello, world!");
    }
}
