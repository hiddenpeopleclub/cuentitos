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

    fn parse(&self, input: &str, context: &mut ParserContext) -> Result<Self::Output, Self::Error> {
        context.current_line += 1;
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
        let initial_line = context.current_line;

        let result = parser.parse("Hello, world!", &mut context).unwrap();
        assert_eq!(result.string, "Hello, world!");
        assert_eq!(context.current_line, initial_line + 1);
    }
}
