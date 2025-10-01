use crate::line_parser;
use cuentitos_common::*;

pub struct SectionParser;

impl SectionParser {
    pub fn parse(
        line: line_parser::Line,
        _level: usize,
    ) -> Option<(Vec<Block>, Vec<std::string::String>)> {
        // For sections, we need to check the raw text, not the parsed result
        // because sections can be indented and we need to preserve that
        let raw_trimmed = line.raw_text.trim();

        // Check if the trimmed text starts with '#'
        if !raw_trimmed.starts_with('#') {
            return None;
        }

        // Now parse to get the indentation level
        let result = line_parser::parse(line).ok()?;

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
                return None;
            }
            (parts[0].trim().to_string(), parts[1].trim().to_string())
        } else if !rest.is_empty() {
            // Use the display name as the ID when no ID is provided
            (rest.to_string(), rest.to_string())
        } else {
            // Section without title is an error
            return None;
        };

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

        let result = SectionParser::parse(line, 0).unwrap();
        let (blocks, _) = result;

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

        let result = SectionParser::parse(line, 1).unwrap();
        let (blocks, _) = result;

        // Section level is determined by hash count, not indentation
        // So a '#' section is always level 0 regardless of indentation
        assert_eq!(blocks[0].level, 0);
    }
}
