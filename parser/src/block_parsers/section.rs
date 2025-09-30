use crate::line_parser;
use cuentitos_common::*;

pub struct SectionParser;

impl SectionParser {
    pub fn parse(
        line: line_parser::Line,
        _level: usize,
    ) -> Option<(Vec<Block>, Vec<std::string::String>)> {
        let result = line_parser::parse(line).ok()?;

        // Check if the trimmed text starts with '#'
        if !result.string.starts_with('#') {
            return None;
        }

        // Count the number of '#' symbols
        let hash_count = result.string.chars().take_while(|&c| c == '#').count();

        // The rest of the text after the '#' symbols
        let rest = result.string[hash_count..].trim();

        // Parse the format: "section_id: Display Name"
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() != 2 {
            return None;
        }

        let section_id = parts[0].trim().to_string();
        let display_name = parts[1].trim().to_string();

        // Section depth is determined by the number of '#' symbols
        // 1 '#' = level 0, 2 '#' = level 1, 3 '#' = level 2, etc.
        // The indentation of the line itself is ignored for sections
        let section_level = hash_count - 1;

        let block = Block::new(
            BlockType::Section {
                id: section_id,
                display_name,
            },
            None,
            section_level,
        );

        Some((vec![block], vec![]))
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

        let result = SectionParser::parse(line, 0).unwrap();
        let (blocks, strings) = result;

        assert_eq!(blocks.len(), 1);
        assert_eq!(strings.len(), 0);

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

        let result = SectionParser::parse(line, 0).unwrap();
        let (blocks, _) = result;

        match &blocks[0].block_type {
            BlockType::Section { id, display_name } => {
                assert_eq!(id, "subsection_1");
                assert_eq!(display_name, "First Subsection");
            }
            _ => panic!("Expected Section block type"),
        }
        assert_eq!(blocks[0].level, 1);
    }

    #[test]
    fn test_parse_deep_subsection() {
        let line = line_parser::Line {
            raw_text: "### subsubsection_1: Deep Nested Section",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0).unwrap();
        let (blocks, _) = result;

        match &blocks[0].block_type {
            BlockType::Section { id, display_name } => {
                assert_eq!(id, "subsubsection_1");
                assert_eq!(display_name, "Deep Nested Section");
            }
            _ => panic!("Expected Section block type"),
        }
        assert_eq!(blocks[0].level, 2);
    }

    #[test]
    fn test_parse_non_section() {
        let line = line_parser::Line {
            raw_text: "Just a regular line",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_section_without_colon() {
        let line = line_parser::Line {
            raw_text: "# section without colon",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_indented_section() {
        let line = line_parser::Line {
            raw_text: "  # section_1: First Section",
            file_path: None,
            line_number: 1,
        };

        let result = SectionParser::parse(line, 1).unwrap();
        let (blocks, _) = result;

        // Section level is determined by hash count, not indentation
        // So a '#' section is always level 0 regardless of indentation
        assert_eq!(blocks[0].level, 0);
    }
}
