use crate::line_parser;
use cuentitos_common::*;

pub struct StringParser;

impl StringParser {
    pub fn parse(line: line_parser::Line, level: usize) -> Option<(Vec<Block>, Vec<std::string::String>)> {
        let result = line_parser::parse(line).ok()?;

        if result.string.is_empty() {
            return None;
        }

        // Create a temporary string ID - the actual ID will be assigned by the parser
        let block = Block::new(BlockType::String(0), None, level);
        Some((vec![block], vec![result.string]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_basic_string() {
        let line = line_parser::Line {
            raw_text: "Hello world",
            file_path: None,
            line_number: 1,
        };

        let result = StringParser::parse(line, 0).unwrap();
        let (blocks, strings) = result;

        assert_eq!(blocks.len(), 1);
        assert_eq!(strings.len(), 1);
        assert_eq!(strings[0], "Hello world");

        if let BlockType::String(id) = blocks[0].block_type {
            assert_eq!(id, 0); // Should be 0 as it's temporary
        } else {
            panic!("Expected String block type");
        }
        assert_eq!(blocks[0].level, 0);
    }

    #[test]
    fn test_parse_empty_line() {
        let line = line_parser::Line {
            raw_text: "",
            file_path: None,
            line_number: 1,
        };

        let result = StringParser::parse(line, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_with_indentation() {
        let line = line_parser::Line {
            raw_text: "  Indented text",
            file_path: None,
            line_number: 1,
        };

        let result = StringParser::parse(line, 2).unwrap();
        let (blocks, strings) = result;

        assert_eq!(blocks[0].level, 2);
        assert_eq!(strings[0], "Indented text");
    }

    #[test]
    fn test_parse_with_file_path() {
        let line = line_parser::Line {
            raw_text: "Test with file",
            file_path: Some(PathBuf::from("test.txt")),
            line_number: 42,
        };

        let result = StringParser::parse(line, 0).unwrap();
        let (blocks, strings) = result;

        assert_eq!(strings[0], "Test with file");
        assert_eq!(blocks[0].level, 0);
    }
}
