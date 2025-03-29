use cuentitos_common::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::parsers::{FeatureParser, ParserContext};
use crate::parsers::line_parser::LineParser;
use crate::parsers::section_parser::SectionParser;
use crate::ParseError;

pub struct Parser {
    last_block_at_level: Vec<BlockId>,
    last_section_at_level: Vec<BlockId>,  // Track the last section block at each level
    current_line: usize,
    file_path: PathBuf,
    section_map: HashMap<(String, usize), (BlockId, usize)>, // (section_name, level) -> (block_id, line_number)
    line_parser: LineParser,
    section_parser: SectionParser,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            last_block_at_level: Vec::new(),
            last_section_at_level: Vec::new(),
            current_line: 0,
            file_path: PathBuf::from("<unknown>"),
            section_map: HashMap::new(),
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
        }
    }

    pub fn with_file<P: Into<PathBuf>>(file_path: P) -> Self {
        Parser {
            last_block_at_level: Vec::new(),
            last_section_at_level: Vec::new(),
            current_line: 0,
            file_path: file_path.into(),
            section_map: HashMap::new(),
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
        }
    }

    fn parse_indentation<'a>(&self, line: &'a str) -> Result<(usize, &'a str), ParseError> {
        let spaces = line.chars().take_while(|c| *c == ' ').count();

        // Check if indentation is valid (multiple of 2)
        if spaces % 2 != 0 {
            return Err(ParseError::InvalidIndentation {
                file: self.file_path.clone(),
                line: self.current_line,
                spaces,
            });
        }

        Ok((spaces / 2, &line[spaces..]))
    }

    pub fn parse(&mut self, script: &str) -> Result<Database, ParseError> {
        self.current_line = 0;
        let mut db = Database::new();
        let mut context = ParserContext::new();
        context.file_path = Some(self.file_path.clone());

        // Add START block
        let start_id = db.add_block(Block::new(BlockType::Start, None, 0));
        self.last_block_at_level.push(start_id);
        self.last_section_at_level.push(start_id);

        for line in script.lines() {
            self.current_line += 1;
            context.current_line = self.current_line;

            if line.trim().is_empty() {
                continue;
            }

            let (level, content) = self.parse_indentation(line)?;
            dbg!(&line, level, &content);

            if content.starts_with('#') {
                if let Some(section_result) = self.section_parser.parse(content, &mut context)? {
                    dbg!(&section_result.title, section_result.level, &self.last_section_at_level);

                    // Check for orphaned sub-sections
                    if section_result.level > 0 {
                        // For a sub-section at level N, we need a section at level N-1
                        let parent_level = section_result.level - 1;

                        // Ensure we have enough levels tracked
                        while self.last_section_at_level.len() <= parent_level {
                            self.last_section_at_level.push(start_id);
                        }

                        let parent_id = self.last_section_at_level[parent_level];
                        if !matches!(db.blocks[parent_id].block_type, BlockType::Section(_)) {
                            dbg!("No valid parent section found", &section_result.title, section_result.level, &self.last_section_at_level);
                            return Err(ParseError::OrphanedSubSection {
                                file: self.file_path.clone(),
                                line: self.current_line,
                            });
                        }
                    }

                    // Get parent section name if this is a sub-section
                    let parent_name = if section_result.level > 0 {
                        let parent_id = self.last_section_at_level[section_result.level - 1];
                        dbg!("Parent section info", section_result.level - 1, parent_id);
                        if let BlockType::Section(string_id) = db.blocks[parent_id].block_type {
                            Some(db.strings[string_id].clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Check for duplicate section names at the same level under the same parent
                    if let Some((_, previous_line)) = self.section_map.get(&(section_result.title.clone(), section_result.level)) {
                        let parent_name = parent_name.unwrap_or_else(|| "<root>".to_string());
                        return Err(ParseError::DuplicateSectionName {
                            file: self.file_path.clone(),
                            line: self.current_line,
                            name: section_result.title.clone(),
                            parent: parent_name,
                            previous_line: *previous_line,
                        });
                    }

                    // Create section block
                    let string_id = db.add_string(section_result.title.clone());
                    let parent_id = if section_result.level == 0 {
                        Some(start_id)
                    } else {
                        Some(self.last_section_at_level[section_result.level - 1])
                    };

                    let block = Block::new(BlockType::Section(string_id), parent_id, section_result.level);
                    let block_id = db.add_block(block);
                    dbg!("Added section block", &section_result.title, block_id, parent_id);

                    // Update section map and last block at level
                    self.section_map.insert((section_result.title, section_result.level), (block_id, self.current_line));

                    // Extend last_section_at_level if needed and update the current level
                    while self.last_section_at_level.len() <= section_result.level {
                        self.last_section_at_level.push(block_id);
                    }
                    self.last_section_at_level[section_result.level] = block_id;
                    dbg!("Updated last_section_at_level", &self.last_section_at_level);

                    // Also update last_block_at_level
                    while self.last_block_at_level.len() <= section_result.level {
                        self.last_block_at_level.push(block_id);
                    }
                    self.last_block_at_level[section_result.level] = block_id;
                }
            } else {
                let line_result = self.line_parser.parse(content, &mut context)?;
                let string_id = db.add_string(line_result.string);

                // For text blocks, we need to ensure they're properly parented to the last block at the previous level
                let parent_id = if level == 0 {
                    Some(start_id)
                } else if level <= self.last_block_at_level.len() {
                    // Get the last block at the previous level as the parent
                    Some(self.last_block_at_level[level - 1])
                } else {
                    // If we're at a level beyond what we have, use the last available block
                    Some(self.last_block_at_level[self.last_block_at_level.len() - 1])
                };

                let block = Block::new(BlockType::String(string_id), parent_id, level);
                let block_id = db.add_block(block);
                dbg!("Added string block", block_id, parent_id, level);

                // Update last_block_at_level for the current level if needed
                while self.last_block_at_level.len() <= level {
                    self.last_block_at_level.push(block_id);
                }
                self.last_block_at_level[level] = block_id;
                dbg!("Updated last_block_at_level for text", &self.last_block_at_level);
            }
        }

        // Add END block
        db.add_block(Block::new(BlockType::End, Some(start_id), 0));

        Ok(db)
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

        let mut parser = Parser::new();
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

        let mut parser = Parser::new();
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
        let mut parser = Parser::new();
        let result = parser.parse("   Hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_empty_lines() {
        let mut parser = Parser::new();
        let database = parser.parse("\n\nHello\n\n").unwrap();
        assert_eq!(database.blocks.len(), 3);
        assert_eq!(database.strings.len(), 1);
    }

    #[test]
    fn test_invalid_indentation_with_file() {
        let mut parser = Parser::with_file("test.md");
        let result = parser.parse("   Hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_sections() {
        let mut parser = Parser::with_file("test.cuentitos");
        let script = "\
# Main Section
This is text in the main section
  ## First Sub-section
  This is text in the first sub-section
    ### Deep Sub-section
    This is text in a deep sub-section
  ## Second Sub-section
  This is text in the second sub-section

# Another Main Section
This is text in another main section
  ## Its Sub-section
  This is text in its sub-section";

        let database = parser.parse(script).unwrap();

        // Verify we have the expected number of blocks:
        // START + END + 2 root sections + 3 sub-sections + 1 deep sub-section + 6 text blocks
        assert_eq!(database.blocks.len(), 13);

        // Find the block IDs for each section
        let find_section = |title: &str| -> BlockId {
            database.blocks.iter()
                .position(|block| matches!(block.block_type, BlockType::Section(id) if database.strings[id] == title))
                .unwrap()
        };

        let main_section = find_section("Main Section");
        let first_sub = find_section("First Sub-section");
        let deep_sub = find_section("Deep Sub-section");
        let second_sub = find_section("Second Sub-section");
        let another_main = find_section("Another Main Section");
        let its_sub = find_section("Its Sub-section");

        // Verify section levels
        assert_eq!(database.blocks[main_section].level, 0);
        assert_eq!(database.blocks[first_sub].level, 1);
        assert_eq!(database.blocks[deep_sub].level, 2);
        assert_eq!(database.blocks[second_sub].level, 1);
        assert_eq!(database.blocks[another_main].level, 0);
        assert_eq!(database.blocks[its_sub].level, 1);

        // Verify parent-child relationships
        assert_eq!(database.blocks[main_section].parent_id, Some(0)); // Main Section -> START
        assert_eq!(database.blocks[first_sub].parent_id, Some(main_section)); // First Sub-section -> Main Section
        assert_eq!(database.blocks[deep_sub].parent_id, Some(first_sub)); // Deep Sub-section -> First Sub-section
        assert_eq!(database.blocks[second_sub].parent_id, Some(main_section)); // Second Sub-section -> Main Section
        assert_eq!(database.blocks[another_main].parent_id, Some(0)); // Another Main Section -> START
        assert_eq!(database.blocks[its_sub].parent_id, Some(another_main)); // Its Sub-section -> Another Main Section

        // Verify text blocks are present with correct content
        let text_blocks: Vec<_> = database.blocks.iter()
            .filter_map(|block| {
                if let BlockType::String(id) = block.block_type {
                    Some((block.level, database.strings[id].as_str()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(text_blocks, vec![
            (0, "This is text in the main section"),
            (1, "This is text in the first sub-section"),
            (2, "This is text in a deep sub-section"),
            (1, "This is text in the second sub-section"),
            (0, "This is text in another main section"),
            (1, "This is text in its sub-section"),
        ]);
    }

    #[test]
    fn test_orphaned_subsection() {
        let mut parser = Parser::with_file("test.cuentitos");
        let script = "  ## Orphaned Sub-section\n  This should cause an error\n\n# Valid Section\nThis is text in a valid section\n  ## Valid Sub-section\n  This is valid";

        let result = parser.parse(script);
        assert!(result.is_err());

        match result {
            Err(ParseError::OrphanedSubSection { file, line }) => {
                assert_eq!(file.to_str().unwrap(), "test.cuentitos");
                assert_eq!(line, 1);
            }
            _ => panic!("Expected OrphanedSubSection error"),
        }
    }
}
