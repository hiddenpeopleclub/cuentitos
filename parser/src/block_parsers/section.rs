use crate::line_parser;
use crate::ParseError;
use cuentitos_common::*;
use std::path::PathBuf;

pub struct SectionParser;

impl SectionParser {
    pub fn parse(
        line: line_parser::Line,
        _level: usize,
        file_path: Option<PathBuf>,
        line_number: usize,
    ) -> Result<Option<(Vec<Block>, Vec<std::string::String>, usize)>, ParseError> {
        // For sections, we need to check the raw text, not the parsed result
        // because sections can be indented and we need to preserve that
        let raw_trimmed = line.raw_text.trim();

        // Check if the trimmed text starts with '#'
        if !raw_trimmed.starts_with('#') {
            return Ok(None);
        }

        // Now parse to get the indentation level
        let result = line_parser::parse(line).map_err(|e| e)?;

        // Count the number of '#' symbols
        let hash_count = result.string.chars().take_while(|&c| c == '#').count();

        // The rest of the text after the '#' symbols
        let rest = result.string[hash_count..].trim();

        // Parse the format - can be either:
        // "section_id: Display Name" (with ID)
        // "Display Name" (without ID)
        let (section_id, display_name) = if rest.contains(':') {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Ok(None);
            }
            (parts[0].trim().to_string(), parts[1].trim().to_string())
        } else if !rest.is_empty() {
            // Use the display name as the ID when no ID is provided
            (rest.to_string(), rest.to_string())
        } else {
            // Section without title is an error
            return Err(ParseError::SectionWithoutTitle {
                file: file_path,
                line: line_number,
            });
        };

        // Note: Orphaned sub-section validation needs to be done at the parser level
        // where we have context about previous sections

        // Section depth is determined by both indentation AND the number of '#' symbols
        // The actual level is the indentation level from the line
        let section_level = result.indentation_level;

        let block = Block::new(
            BlockType::Section {
                id: section_id,
                display_name,
            },
            None,
            section_level,
        );

        // Return block, strings, and hash_count
        Ok(Some((vec![block], vec![], hash_count)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_basic_section() {
        let line = line_parser::Line {
            raw_text: "# section_1: First Section",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0, None, 1).unwrap().unwrap();
        let (blocks, strings, hash_count) = result;

        assert_eq!(blocks.len(), 1);
        assert_eq!(strings.len(), 0);
        assert_eq!(hash_count, 1);

        match &blocks[0].block_type {
            BlockType::Section { id, display_name } => {
                assert_eq!(id, "section_1");
                assert_eq!(display_name, "First Section");
            }
            _ => panic!("Expected Section block type"),
        }
        assert_eq!(blocks[0].level, 0);
    }

    #[test]
    fn test_parse_subsection() {
        let line = line_parser::Line {
            raw_text: "## subsection_1: First Subsection",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0, None, 1).unwrap().unwrap();
        let (blocks, _, hash_count) = result;

        assert_eq!(hash_count, 2);
        match &blocks[0].block_type {
            BlockType::Section { id, display_name } => {
                assert_eq!(id, "subsection_1");
                assert_eq!(display_name, "First Subsection");
            }
            _ => panic!("Expected Section block type"),
        }
        assert_eq!(blocks[0].level, 0);
    }

    #[test]
    fn test_parse_deep_subsection() {
        let line = line_parser::Line {
            raw_text: "### subsubsection_1: Deep Nested Section",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0, None, 1).unwrap().unwrap();
        let (blocks, _, hash_count) = result;

        assert_eq!(hash_count, 3);
        match &blocks[0].block_type {
            BlockType::Section { id, display_name } => {
                assert_eq!(id, "subsubsection_1");
                assert_eq!(display_name, "Deep Nested Section");
            }
            _ => panic!("Expected Section block type"),
        }
        assert_eq!(blocks[0].level, 0);
    }

    #[test]
    fn test_parse_non_section() {
        let line = line_parser::Line {
            raw_text: "Just a regular line",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0, None, 1).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_section_without_colon() {
        let line = line_parser::Line {
            raw_text: "# section without colon",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0, None, 1).unwrap().unwrap();
        let (blocks, _, hash_count) = result;

        assert_eq!(hash_count, 1);
        match &blocks[0].block_type {
            BlockType::Section { id, display_name } => {
                // When no colon, the title is used as both ID and display name
                assert_eq!(id, "section without colon");
                assert_eq!(display_name, "section without colon");
            }
            _ => panic!("Expected Section block type"),
        }
    }

    #[test]
    fn test_parse_indented_section() {
        let line = line_parser::Line {
            raw_text: "  # section_1: First Section",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 1, None, 1).unwrap().unwrap();
        let (blocks, _, hash_count) = result;

        assert_eq!(hash_count, 1);
        // Section level is determined by indentation
        assert_eq!(blocks[0].level, 1);
    }

    #[test]
    fn test_parse_empty_section_title() {
        let line = line_parser::Line {
            raw_text: "#",
            file_path: None,
            line_number: 4,
        };

        let result = SectionParser::parse(line, 0, None, 4);
        match result {
            Err(ParseError::SectionWithoutTitle { .. }) => {},
            _ => panic!("Expected SectionWithoutTitle error"),
        }
    }
}
