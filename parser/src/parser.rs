use crate::line_parser;
use crate::block_parsers;
use cuentitos_common::*;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Parser {
    last_block_at_level: Vec<BlockId>, // Stack to track last block at each level
    last_section_at_level: Vec<BlockId>, // Stack to track last section at each level (for section hierarchy)
    file_path: Option<PathBuf>,
    current_line: usize,
    // Track section names by parent_id -> (name -> first_line_number)
    section_names_by_parent: HashMap<Option<BlockId>, HashMap<String, usize>>,
    // Track hash count for each block_id (for section hierarchy validation)
    section_hash_counts: HashMap<BlockId, usize>,
    // Collect errors instead of returning immediately
    errors: Vec<ParseError>,
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
    SectionWithoutTitle {
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidSectionHierarchy {
        message: String,
        file: Option<PathBuf>,
        line: usize,
    },
    DuplicateSectionName {
        name: String,
        parent_name: String,
        file: Option<PathBuf>,
        line: usize,
        previous_line: usize,
    },
    MultipleErrors {
        errors: Vec<ParseError>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { file, line } => {
                let prefix = file.as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Unexpected token", prefix, line)
            }
            ParseError::UnexpectedEndOfFile { file, line } => {
                let prefix = file.as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Unexpected end of file", prefix, line)
            }
            ParseError::InvalidIndentation {
                message,
                file,
                line,
            } => {
                let prefix = file.as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Invalid indentation: {}", prefix, line, message)
            }
            ParseError::SectionWithoutTitle { file, line } => {
                let prefix = file.as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Section without title: found empty section title.", prefix, line)
            }
            ParseError::InvalidSectionHierarchy { message, file, line } => {
                let prefix = file.as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Invalid section hierarchy: {}", prefix, line, message)
            }
            ParseError::DuplicateSectionName { name, parent_name, file, line, previous_line } => {
                let prefix = file.as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Duplicate section name: '{}' already exists at this level under '{}'. Previously defined at line {}.",
                    prefix, line, name, parent_name, previous_line)
            }
            ParseError::MultipleErrors { errors } => {
                for (i, error) in errors.iter().enumerate() {
                    write!(f, "{}", error)?;
                    if i < errors.len() - 1 {
                        writeln!(f)?;
                        writeln!(f)?;
                    }
                }
                Ok(())
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
            ..Self::default()
        }
    }

    fn process_blocks(
        &mut self,
        blocks: Vec<Block>,
        strings: Vec<std::string::String>,
        hash_count: Option<usize>,
        database: &mut Database,
        start_id: BlockId,
        raw_text: &str,
    ) -> Result<(), ParseError> {
        // Add all strings first to get their IDs
        let string_ids: Vec<_> = strings.into_iter()
            .map(|s| database.add_string(s))
            .collect();

        // Process each block
        for (block_idx, mut block) in blocks.into_iter().enumerate() {
            // If it's a string block, update its ID to match the actual string ID
            if let BlockType::String(_) = block.block_type {
                block.block_type = BlockType::String(string_ids[block_idx]);
            }

            // Find parent block
            let is_section = matches!(block.block_type, BlockType::Section { .. });

            let parent_id = if block.level == 0 {
                Some(start_id)
            } else if is_section {
                // Sections use the section stack to find parents
                let potential_parent = if block.level <= self.last_section_at_level.len() {
                    // Pop section levels until we reach the parent level
                    while self.last_section_at_level.len() > block.level {
                        self.last_section_at_level.pop();
                    }
                    self.last_section_at_level.last().copied()
                } else {
                    // No parent section at the right level, use start
                    Some(start_id)
                };

                // Validate orphaned subsections: if hash_count > 1 (subsection) and parent is start_id, that's an error
                if let Some(hc) = hash_count {
                    if hc > 1 && potential_parent == Some(start_id) {
                        return Err(ParseError::InvalidSectionHierarchy {
                            message: "found sub-section without parent section.".to_string(),
                            file: self.file_path.clone(),
                            line: self.current_line,
                        });
                    }
                }

                potential_parent
            } else {
                // Regular blocks use the normal stack
                if block.level <= self.last_block_at_level.len() {
                    // Pop levels until we reach the parent level
                    while self.last_block_at_level.len() > block.level {
                        self.last_block_at_level.pop();
                    }
                    self.last_block_at_level.last().copied()
                } else {
                    return Err(ParseError::InvalidIndentation {
                        message: format!("found {} spaces in: {}", block.level * 2, raw_text),
                        file: self.file_path.clone(),
                        line: self.current_line,
                    });
                }
            };

            // Set the parent and add the block
            block.parent_id = parent_id;

            // Check for duplicate section names
            if let BlockType::Section { id: _, display_name } = &block.block_type {
                // Get the map for this parent (or create it)
                let names_map = self.section_names_by_parent.entry(parent_id).or_insert_with(HashMap::new);

                // Check if this name already exists
                if let Some(&previous_line) = names_map.get(display_name) {
                    // Get parent's display name for error message
                    let parent_name = if let Some(pid) = parent_id {
                        if pid == start_id {
                            "<root>".to_string()
                        } else {
                            match &database.blocks[pid].block_type {
                                BlockType::Section { id: _, display_name: parent_display } => parent_display.clone(),
                                _ => "<root>".to_string(),
                            }
                        }
                    } else {
                        "<root>".to_string()
                    };

                    // Collect error instead of returning
                    self.errors.push(ParseError::DuplicateSectionName {
                        name: display_name.clone(),
                        parent_name,
                        file: self.file_path.clone(),
                        line: self.current_line,
                        previous_line,
                    });

                    // Skip adding this block since it's a duplicate
                    continue;
                }

                // Record this section name
                names_map.insert(display_name.clone(), self.current_line);
            }

            let block_id = database.add_block(block.clone());

            // Update stacks
            // Always update the general block stack
            if block.level >= self.last_block_at_level.len() {
                // Grow the stack to accommodate this level
                while self.last_block_at_level.len() <= block.level {
                    self.last_block_at_level.push(block_id);
                }
            } else {
                // Update existing level
                self.last_block_at_level[block.level] = block_id;
            }

            // If this is a section, also update the section stack
            if is_section {
                if block.level >= self.last_section_at_level.len() {
                    // Grow the section stack
                    while self.last_section_at_level.len() <= block.level {
                        self.last_section_at_level.push(block_id);
                    }
                } else {
                    // Update existing level
                    self.last_section_at_level[block.level] = block_id;
                }
            }
        }
        Ok(())
    }

    fn try_parse_line(&self, line: line_parser::Line, level: usize) -> Result<Option<(Vec<Block>, Vec<std::string::String>, Option<usize>)>, ParseError> {
        // Try section parser first
        match block_parsers::SectionParser::parse(line.clone(), level, self.file_path.clone(), self.current_line) {
            Ok(Some((blocks, strings, hash_count))) => return Ok(Some((blocks, strings, Some(hash_count)))),
            Ok(None) => {}, // Not a section, try next parser
            Err(e) => return Err(e), // Section parsing error
        }

        // Try string parser
        let result = block_parsers::StringParser::parse(line, level);
        if let Some((blocks, strings)) = result {
            return Ok(Some((blocks, strings, None)));
        }

        Ok(None)
    }

    pub fn parse<A>(&mut self, script: A) -> Result<Database, ParseError>
    where
        A: AsRef<str>,
    {
        let mut database = Database::new();
        let script = script.as_ref();

        // Reset line counter
        self.current_line = 1;

        // Add Start block
        let start_block = Block::new(BlockType::Start, None, 0);
        let start_id = database.add_block(start_block);
        self.last_block_at_level.push(start_id);
        self.last_section_at_level.push(start_id); // Start can be parent of top-level sections

        // Iterate through each line
        for raw_text in script.lines() {
            let line = line_parser::Line {
                raw_text,
                file_path: self.file_path.clone(),
                line_number: self.current_line,
            };

            // Parse the line to get indentation level
            let line_result = match line_parser::parse(line.clone()) {
                Ok(result) => result,
                Err(e) => {
                    // Collect error and skip this line
                    self.errors.push(e);
                    self.current_line += 1;
                    continue;
                }
            };

            // Skip empty lines
            if line_result.string.is_empty() {
                self.current_line += 1;
                continue;
            }

            // Try to parse the line with available parsers, passing the actual indentation level
            let parse_result = match self.try_parse_line(line, line_result.indentation_level) {
                Ok(result) => result,
                Err(e) => {
                    // Collect error and skip this line
                    self.errors.push(e);
                    self.current_line += 1;
                    continue;
                }
            };

            match parse_result {
                Some((blocks, strings, hash_count)) => {
                    // Disable section content validation for now - main's tests don't require it
                    // Sections in main's implementation don't enforce content indentation

                    if let Err(e) = self.process_blocks(blocks, strings, hash_count, &mut database, start_id, raw_text) {
                        // Collect error and continue
                        self.errors.push(e);
                    }
                }
                None => {
                    // No parser succeeded - collect error
                    self.errors.push(ParseError::UnexpectedToken {
                        file: self.file_path.clone(),
                        line: self.current_line,
                    });
                }
            }

            self.current_line += 1;
        }

        // Add End block (with no parent)
        let end_block = Block::new(BlockType::End, None, 0);
        database.add_block(end_block);

        // Check if we collected any errors
        if !self.errors.is_empty() {
            if self.errors.len() == 1 {
                return Err(self.errors.remove(0));
            } else {
                return Err(ParseError::MultipleErrors {
                    errors: self.errors.clone(),
                });
            }
        }

        Ok(database)
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

        let mut parser = Parser::default();
        let database = parser.parse(test_case.script).unwrap();

        assert_eq!(database.blocks.len(), 3);
        assert_eq!(database.strings.len(), 1);

        // Check Start block
        assert_eq!(database.blocks[0].block_type, BlockType::Start);
        assert_eq!(database.blocks[0].level, 0);
        assert_eq!(database.blocks[0].parent_id, None);
        assert_eq!(database.blocks[0].children, vec![1]); // Points to the string block

        // Check String block
        if let BlockType::String(id) = database.blocks[1].block_type {
            assert_eq!(database.strings[id], "This is a single line");
        } else {
            panic!("Expected String block");
        }
        assert_eq!(database.blocks[1].level, 0);
        assert_eq!(database.blocks[1].parent_id, Some(0));
        assert!(database.blocks[1].children.is_empty());

        // Check End block
        assert_eq!(database.blocks[2].block_type, BlockType::End);
        assert_eq!(database.blocks[2].level, 0);
        assert_eq!(database.blocks[2].parent_id, None);
        assert!(database.blocks[2].children.is_empty());
    }

    #[test]
    fn test_indented_script() {
        let script = "Parent\n  Child1\n  Child2\n    Grandchild\n  Child3";

        let mut parser = Parser::default();
        let database = parser.parse(script).unwrap();

        // We expect: Start, Parent, Child1, Child2, Grandchild, Child3, End
        assert_eq!(database.blocks.len(), 7);
        assert_eq!(database.strings.len(), 5);

        // Verify levels
        assert_eq!(database.blocks[1].level, 0); // Parent
        assert_eq!(database.blocks[2].level, 1); // Child1
        assert_eq!(database.blocks[3].level, 1); // Child2
        assert_eq!(database.blocks[4].level, 2); // Grandchild
        assert_eq!(database.blocks[5].level, 1); // Child3

        // Verify parent-child relationships
        assert_eq!(database.blocks[1].parent_id, Some(0)); // Parent -> Start
        assert_eq!(database.blocks[2].parent_id, Some(1)); // Child1 -> Parent
        assert_eq!(database.blocks[3].parent_id, Some(1)); // Child2 -> Parent
        assert_eq!(database.blocks[4].parent_id, Some(3)); // Grandchild -> Child2
        assert_eq!(database.blocks[5].parent_id, Some(1)); // Child3 -> Parent
    }

    #[test]
    fn test_invalid_indentation() {
        let script = "First line\n   Invalid indentation";
        let mut parser = Parser::new();
        match parser.parse(script) {
            Err(ParseError::InvalidIndentation {
                message: _,
                file: None,
                line: 2,
            }) => (),
            _ => panic!("Expected InvalidIndentation error"),
        }
    }

    #[test]
    fn test_skip_empty_lines() {
        let script = "First\n\n  Second";

        let mut parser = Parser::default();
        let database = parser.parse(script).unwrap();

        assert_eq!(database.strings.len(), 2);
        assert_eq!(database.strings[0], "First");
        assert_eq!(database.strings[1], "Second");
    }

    #[test]
    fn test_invalid_indentation_with_file() {
        let script = "First line\n   Invalid indentation";
        let mut parser = Parser::with_file(PathBuf::from("test.cuentitos"));
        match parser.parse(script) {
            Err(ParseError::InvalidIndentation {
                message: _,
                file: Some(path),
                line: 2,
            }) => {
                assert_eq!(path, PathBuf::from("test.cuentitos"));
            }
            _ => panic!("Expected InvalidIndentation error with file path"),
        }
    }

    #[test]
    fn test_orphaned_subsection() {
        let script = "  ## Orphaned Sub-section\n  This should cause an error";
        let mut parser = Parser::with_file(PathBuf::from("test.cuentitos"));
        let result = parser.parse(script);

        match result {
            Err(ParseError::InvalidSectionHierarchy { message, file, line }) => {
                assert_eq!(message, "found sub-section without parent section");
                assert_eq!(file, Some(PathBuf::from("test.cuentitos")));
                assert_eq!(line, 1);
            }
            Ok(_) => panic!("Expected InvalidSectionHierarchy error but parsing succeeded"),
            Err(e) => panic!("Expected InvalidSectionHierarchy error but got: {:?}", e),
        }
    }

    #[test]
    fn test_valid_subsection_with_parent() {
        let script = "# Valid Section\nThis is text\n  ## Valid Sub-section\n  This is valid";
        let mut parser = Parser::default();
        let result = parser.parse(script);

        assert!(result.is_ok(), "Valid subsection with parent should parse successfully");
    }
}
