use crate::ParseError;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Line<'a> {
    pub raw_text: &'a str,
    pub file_path: Option<PathBuf>,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub string: String,
    pub indentation_level: usize,
}

pub fn parse(line: Line) -> Result<ParseResult, ParseError> {
    let raw_text = line.raw_text;

    // Count leading spaces
    let leading_spaces = raw_text.len() - raw_text.trim_start().len();

    // Validate that indentation is in multiples of 2
    if leading_spaces % 2 != 0 {
        return Err(ParseError::InvalidIndentation {
            message: format!("found {} spaces.", leading_spaces),
            file: line.file_path,
            line: line.line_number,
        });
    }

    // Calculate indentation level (2 spaces = 1 level)
    let indentation_level = leading_spaces / 2;

    // Get trimmed text
    let string = raw_text.trim().to_string();

    Ok(ParseResult {
        string,
        indentation_level,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_indentation() {
        let line = Line {
            raw_text: "Hello, world!",
            file_path: None,
            line_number: 1,
        };

        let result = parse(line).unwrap();
        assert_eq!(result.string, "Hello, world!");
        assert_eq!(result.indentation_level, 0);
    }

    #[test]
    fn test_parse_with_indentation() {
        let line = Line {
            raw_text: "  Indented text",
            file_path: None,
            line_number: 1,
        };

        let result = parse(line).unwrap();
        assert_eq!(result.string, "Indented text");
        assert_eq!(result.indentation_level, 1);
    }

    #[test]
    fn test_parse_deep_indentation() {
        let line = Line {
            raw_text: "      Deep indentation",
            file_path: None,
            line_number: 1,
        };

        let result = parse(line).unwrap();
        assert_eq!(result.string, "Deep indentation");
        assert_eq!(result.indentation_level, 3);
    }

    #[test]
    fn test_parse_invalid_indentation() {
        let line = Line {
            raw_text: "   Odd indentation",
            file_path: None,
            line_number: 2,
        };

        match parse(line) {
            Err(ParseError::InvalidIndentation { message: _, file: None, line: 2 }) => (),
            _ => panic!("Expected InvalidIndentation error"),
        }
    }

    #[test]
    fn test_parse_empty_line() {
        let line = Line {
            raw_text: "",
            file_path: None,
            line_number: 1,
        };

        let result = parse(line).unwrap();
        assert_eq!(result.string, "");
        assert_eq!(result.indentation_level, 0);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let line = Line {
            raw_text: "    ",
            file_path: None,
            line_number: 1,
        };

        let result = parse(line).unwrap();
        assert_eq!(result.string, "");
        assert_eq!(result.indentation_level, 2);
    }
}
