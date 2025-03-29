use crate::parsers::{FeatureParser, ParserContext, line_parser::LineParser, section_parser::SectionParser};
use cuentitos_common::*;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Parser {
    last_block_at_level: Vec<BlockId>, // Stack to track last block at each level
    file_path: Option<PathBuf>,
    line_parser: LineParser,
    section_parser: SectionParser,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        file: Option<PathBuf>,
        line: usize,
    },
    UnexpectedEndOfFile {
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidIndentation {
        message: String,
        file: Option<PathBuf>,
        line: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { file: _, line } => {
                write!(f, "{}: ERROR: Unexpected token", line)
            }
            ParseError::UnexpectedEndOfFile { file: _, line } => {
                write!(f, "{}: ERROR: Unexpected end of file", line)
            }
            ParseError::InvalidIndentation {
                message,
                file: _,
                line,
            } => {
                write!(f, "{}: ERROR: Invalid indentation: {}", line, message)
            }
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(file_path: PathBuf) -> Self {
        Self {
            file_path: Some(file_path),
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
            last_block_at_level: vec![],
        }
    }

    pub fn parse(&self, script: &str) -> Result<Database, ParseError> {
        let mut context = ParserContext::new();
        if let Some(file_path) = &self.file_path {
            context.file_path = Some(file_path.clone());
        }

        let mut last_block_at_level = vec![0]; // Start block is at level 0
        let mut last_section_at_level = vec![0]; // Start block is at level 0

        // Add START block
        context.database.blocks.push(Block::new(BlockType::Start, None, 0));

        for line in script.lines() {
            context.current_line += 1;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse indentation
            let (level, content) = self.parse_indentation(line, &context)?;
            context.current_level = level;

            // Try to parse as section first
            if let Some(section_result) = self.section_parser.parse(content, &mut context)? {
                // For sections, level is determined by the number of # symbols
                let section_level = section_result.level;

                // Truncate deeper levels when we go back up
                last_section_at_level.truncate(section_level + 1);

                // Find parent - it's either:
                // 1. START (0) for level 0 sections
                // 2. The previous section at the same level (for siblings)
                // 3. The last section at the level above (for first section at this level)
                let parent_id = if section_level == 0 {
                    Some(0) // Root sections are children of START
                } else if last_section_at_level.len() > section_level && context.database.blocks[last_section_at_level[section_level]].level == section_level {
                    // If there's already a section at this level, use it as the parent
                    let prev_section = last_section_at_level[section_level];
                    Some(prev_section)
                } else {
                    // First section at this level, parent is the last section at level above
                    let parent_level = section_level - 1;
                    Some(last_section_at_level[parent_level])
                };

                // Create section block
                let string_id = context.database.add_string(section_result.title);
                let block_id = context.database.blocks.len();
                let block = Block::new(BlockType::Section(string_id), parent_id, section_level);

                // Update parent's children array
                if let Some(parent_id) = parent_id {
                    if !context.database.blocks[parent_id].children.contains(&block_id) {
                        context.database.blocks[parent_id].children.push(block_id);
                    }
                }

                context.database.blocks.push(block);

                // Update tracking arrays
                while last_block_at_level.len() <= section_level {
                    last_block_at_level.push(block_id);
                }
                last_block_at_level[section_level] = block_id;

                // Update section tracking array
                while last_section_at_level.len() <= section_level {
                    last_section_at_level.push(block_id);
                }
                last_section_at_level[section_level] = block_id;
            } else {
                // Try to parse as regular line
                let line_result = self.line_parser.parse(content, &mut context)?;
                // For regular lines, level is determined by indentation
                // Truncate tracking array to current level
                last_block_at_level.truncate(level + 1);

                // Find parent - it's the last block at the level above
                let parent_id = if level == 0 {
                    Some(0) // Root blocks are children of START
                } else {
                    Some(last_block_at_level[level - 1])
                };

                // Create string block
                let string_id = context.database.add_string(line_result.string);
                let block_id = context.database.blocks.len();
                let block = Block::new(BlockType::String(string_id), parent_id, level);

                // Update parent's children array
                if let Some(parent_id) = parent_id {
                    if !context.database.blocks[parent_id].children.contains(&block_id) {
                        context.database.blocks[parent_id].children.push(block_id);
                    }
                }

                context.database.blocks.push(block);

                // Update tracking array
                while last_block_at_level.len() <= level {
                    last_block_at_level.push(block_id);
                }
                last_block_at_level[level] = block_id;
            }
        }

        // Add END block
        let end_block = Block::new(BlockType::End, Some(0), 0);
        context.database.blocks.push(end_block);

        Ok(context.database)
    }

    fn parse_indentation<'a>(&self, line: &'a str, context: &ParserContext) -> Result<(usize, &'a str), ParseError> {
        let spaces = line.chars().take_while(|c| *c == ' ').count();

        // Check if indentation is valid (multiple of 2)
        if spaces % 2 != 0 {
            return Err(ParseError::InvalidIndentation {
                message: format!("found {} spaces.", spaces),
                file: self.file_path.clone(),
                line: context.current_line,
            });
        }

        Ok((spaces / 2, &line[spaces..]))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cuentitos_common::test_case::TestCase;

    #[test]
    fn test_single_line_script() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000001-single-line-and-end.md"),
            "single-line.md",
        );

        let parser = Parser::new();
        let database = parser.parse(&test_case.script).unwrap();

        assert_eq!(database.blocks.len(), 3);
        assert_eq!(database.strings.len(), 1);
    }

    #[test]
    fn test_indented_script() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000009-nested-strings-with-siblings.md"),
            "nested-strings.md",
        );

        let parser = Parser::new();
        let database = parser.parse(&test_case.script).unwrap();

        assert_eq!(database.blocks.len(), 10); // START + 8 strings + END
        assert_eq!(database.strings.len(), 8);

        // Verify levels
        assert_eq!(database.blocks[1].level, 0); // Parent
        assert_eq!(database.blocks[2].level, 1); // First child
        assert_eq!(database.blocks[3].level, 2); // First grandchild
        assert_eq!(database.blocks[4].level, 2); // Second grandchild
        assert_eq!(database.blocks[5].level, 1); // Second child
        assert_eq!(database.blocks[6].level, 2); // Third grandchild
        assert_eq!(database.blocks[7].level, 2); // Fourth grandchild
        assert_eq!(database.blocks[8].level, 1); // Third child

        // Verify parent-child relationships
        assert_eq!(database.blocks[1].parent_id, Some(0)); // Parent -> START
        assert_eq!(database.blocks[2].parent_id, Some(1)); // First child -> Parent
        assert_eq!(database.blocks[3].parent_id, Some(2)); // First grandchild -> First child
        assert_eq!(database.blocks[4].parent_id, Some(2)); // Second grandchild -> First child
        assert_eq!(database.blocks[5].parent_id, Some(1)); // Second child -> Parent
        assert_eq!(database.blocks[6].parent_id, Some(5)); // Third grandchild -> Second child
        assert_eq!(database.blocks[7].parent_id, Some(5)); // Fourth grandchild -> Second child
        assert_eq!(database.blocks[8].parent_id, Some(1)); // Third child -> Parent
    }

    #[test]
    fn test_invalid_indentation() {
        let parser = Parser::new();
        let result = parser.parse("   Hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_empty_lines() {
        let parser = Parser::new();
        let database = parser.parse("\n\nHello\n\n").unwrap();

        assert_eq!(database.blocks.len(), 3);
        assert_eq!(database.strings.len(), 1);
    }

    #[test]
    fn test_invalid_indentation_with_file() {
        let parser = Parser::with_file(PathBuf::from("test.cuentitos"));
        let result = parser.parse("   Hello");
        assert!(result.is_err());
    }
}
