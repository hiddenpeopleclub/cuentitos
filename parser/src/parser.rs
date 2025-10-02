use crate::parsers::{FeatureParser, ParserContext, line_parser::LineParser, section_parser::SectionParser};
use cuentitos_common::*;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Parser {
    last_block_at_level: Vec<BlockId>, // Stack to track last block at each level
    last_section_at_level: Vec<BlockId>, // Stack to track last section at each level (for section hierarchy)
    file_path: Option<PathBuf>,
    line_parser: LineParser,
    section_parser: SectionParser,
    // Track section names by parent_id -> (name -> first_line_number)
    section_names_by_parent: HashMap<Option<BlockId>, HashMap<String, usize>>,
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
        Self {
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
            ..Self::default()
        }
    }

    pub fn with_file(file_path: PathBuf) -> Self {
        Self {
            file_path: Some(file_path),
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
            ..Self::default()
        }
    }

    /// Helper to collect an error and skip the current line
    fn collect_error_and_skip(&mut self, error: ParseError, context: &mut ParserContext) {
        self.errors.push(error);
        context.current_line += 1;
    }

    pub fn parse<A>(&mut self, script: A) -> Result<Database, ParseError>
    where
        A: AsRef<str>,
    {
        let mut context = if let Some(file_path) = &self.file_path {
            ParserContext::with_file(file_path.clone())
        } else {
            ParserContext::new()
        };

        // Add Start block
        let start_block = Block::new(BlockType::Start, None, 0);
        let start_id = context.database.add_block(start_block);
        self.last_block_at_level.push(start_id);
        self.last_section_at_level.push(start_id); // Start can be parent of top-level sections

        // Iterate through each line
        for line in script.as_ref().lines() {
            let (level, content) = match self.parse_indentation(line, &context) {
                Ok(result) => result,
                Err(e) => {
                    self.collect_error_and_skip(e, &mut context);
                    continue;
                }
            };

            if content.trim().is_empty() {
                context.current_line += 1;
                continue; // Skip empty lines
            }

            // Try to parse as section first
            let section_result = match self.section_parser.parse(content.trim(), &mut context) {
                Ok(result) => result,
                Err(e) => {
                    self.collect_error_and_skip(e, &mut context);
                    continue;
                }
            };

            if let Some(section_result) = section_result {
                // This is a section
                let parent_id = if level == 0 {
                    Some(start_id)
                } else if level <= self.last_section_at_level.len() {
                    // Pop section levels until we reach the parent level
                    while self.last_section_at_level.len() > level {
                        self.last_section_at_level.pop();
                    }
                    self.last_section_at_level.last().copied()
                } else {
                    Some(start_id)
                };

                // Validate orphaned subsections: if hash_count > 1 (subsection) and parent is start_id, that's an error
                if section_result.hash_count > 1 && parent_id == Some(start_id) {
                    self.collect_error_and_skip(
                        ParseError::InvalidSectionHierarchy {
                            message: "found sub-section without parent section.".to_string(),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        },
                        &mut context,
                    );
                    continue;
                }

                // Check for duplicate section names
                let names_map = self.section_names_by_parent.entry(parent_id).or_insert_with(HashMap::new);

                if let Some(&previous_line) = names_map.get(&section_result.display_name) {
                    // Get parent's display name for error message
                    let parent_name = if let Some(pid) = parent_id {
                        if pid == start_id {
                            "<root>".to_string()
                        } else {
                            match &context.database.blocks[pid].block_type {
                                BlockType::Section { display_name: parent_display, .. } => parent_display.clone(),
                                _ => "<root>".to_string(),
                            }
                        }
                    } else {
                        "<root>".to_string()
                    };

                    // Collect error and skip adding this block
                    self.collect_error_and_skip(
                        ParseError::DuplicateSectionName {
                            name: section_result.display_name.clone(),
                            parent_name,
                            file: self.file_path.clone(),
                            line: context.current_line,
                            previous_line,
                        },
                        &mut context,
                    );
                    continue;
                }

                // Record this section name
                names_map.insert(section_result.display_name.clone(), context.current_line);

                let block = Block::new(
                    BlockType::Section {
                        id: section_result.id,
                        display_name: section_result.display_name,
                    },
                    parent_id,
                    level,
                );
                let block_id = context.database.add_block(block);

                // Update both stacks
                if level >= self.last_block_at_level.len() {
                    self.last_block_at_level.push(block_id);
                } else {
                    self.last_block_at_level[level] = block_id;
                }

                if level >= self.last_section_at_level.len() {
                    self.last_section_at_level.push(block_id);
                } else {
                    self.last_section_at_level[level] = block_id;
                }
            } else {
                // Parse as regular string
                let result = self.line_parser.parse(content.trim(), &mut context)?;

                // Find parent block
                let parent_id = if level == 0 {
                    Some(start_id)
                } else if level <= self.last_block_at_level.len() {
                    // Pop levels until we reach the parent level
                    while self.last_block_at_level.len() > level {
                        self.last_block_at_level.pop();
                    }
                    self.last_block_at_level.last().copied()
                } else {
                    return Err(ParseError::InvalidIndentation {
                        message: format!("found {} spaces in: {}", level * 2, content),
                        file: self.file_path.clone(),
                        line: context.current_line,
                    });
                };

                // Create new block
                let string_id = context.database.add_string(result.string);
                let block = Block::new(BlockType::String(string_id), parent_id, level);
                let block_id = context.database.add_block(block);

                // Update last block at this level
                if level >= self.last_block_at_level.len() {
                    self.last_block_at_level.push(block_id);
                } else {
                    self.last_block_at_level[level] = block_id;
                }
            }

            // Increment line counter after processing each non-empty line
            context.current_line += 1;
        }

        // Add End block (with no parent)
        let end_block = Block::new(BlockType::End, None, 0);
        context.database.add_block(end_block);

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
}
