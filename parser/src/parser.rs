use crate::parsers::{
    go_to_section_and_back_parser::GoToSectionAndBackParser,
    go_to_section_parser::GoToSectionParser, line_parser::LineParser,
    section_parser::SectionParser, FeatureParser, ParserContext,
};
use cuentitos_common::*;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Parser {
    last_block_at_level: Vec<BlockId>, // Stack to track last block at each level
    last_section_at_level: Vec<BlockId>, // Stack to track last section at each level (for section hierarchy)
    file_path: Option<PathBuf>,
    line_parser: LineParser,
    section_parser: SectionParser,
    go_to_section_and_back_parser: GoToSectionAndBackParser,
    go_to_section_parser: GoToSectionParser,
    // Track section names by parent_id -> (name -> first_line_number)
    section_names_by_parent: HashMap<Option<BlockId>, HashMap<String, usize>>,
    // Collect errors instead of returning immediately
    errors: Vec<ParseError>,
    // Collect warnings
    warnings: Vec<Warning>,
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
    InvalidGoToSection {
        message: String,
        file: Option<PathBuf>,
        line: usize,
    },
    SectionNotFound {
        path: String,
        file: Option<PathBuf>,
        line: usize,
    },
    NavigationAboveRoot {
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidSectionName {
        message: String,
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    EmptySection {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    MultipleErrors {
        errors: Vec<ParseError>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Unexpected token", prefix, line)
            }
            ParseError::UnexpectedEndOfFile { file, line } => {
                let prefix = file
                    .as_ref()
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
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(
                    f,
                    "{}:{}: ERROR: Invalid indentation: {}",
                    prefix, line, message
                )
            }
            ParseError::SectionWithoutTitle { file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(
                    f,
                    "{}:{}: ERROR: Section without title: found empty section title.",
                    prefix, line
                )
            }
            ParseError::InvalidSectionHierarchy {
                message,
                file,
                line,
            } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(
                    f,
                    "{}:{}: ERROR: Invalid section hierarchy: {}",
                    prefix, line, message
                )
            }
            ParseError::DuplicateSectionName {
                name,
                parent_name,
                file,
                line,
                previous_line,
            } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Duplicate section name: '{}' already exists at this level under '{}'. Previously defined at line {}.",
                    prefix, line, name, parent_name, previous_line)
            }
            ParseError::InvalidGoToSection {
                message,
                file,
                line,
            } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: {}", prefix, line, message)
            }
            ParseError::SectionNotFound { path, file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: Section not found: {}", prefix, line, path)
            }
            ParseError::NavigationAboveRoot { file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(
                    f,
                    "{}:{}: ERROR: Cannot navigate above root level",
                    prefix, line
                )
            }
            ParseError::InvalidSectionName {
                message,
                name,
                file,
                line,
            } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(f, "{}:{}: ERROR: {}: {}", prefix, line, message, name)
            }
            ParseError::EmptySection { name, file, line } => {
                let prefix = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("test.cuentitos");
                write!(
                    f,
                    "{}:{}: ERROR: Section must contain at least one block: {}",
                    prefix, line, name
                )
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
            go_to_section_parser: GoToSectionParser::new(),
            ..Self::default()
        }
    }

    pub fn with_file(file_path: PathBuf) -> Self {
        Self {
            file_path: Some(file_path),
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
            go_to_section_and_back_parser: GoToSectionAndBackParser::new(),
            go_to_section_parser: GoToSectionParser::new(),
            ..Self::default()
        }
    }

    /// Helper to collect an error and skip the current line
    fn collect_error_and_skip(&mut self, error: ParseError, context: &mut ParserContext) {
        self.errors.push(error);
        context.current_line += 1;
    }

    pub fn parse<A>(&mut self, script: A) -> Result<(Database, Vec<Warning>), ParseError>
    where
        A: AsRef<str>,
    {
        // Clear warnings from previous parse
        self.warnings.clear();

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
            // Skip comment lines
            if Self::is_comment(line) {
                context.current_line += 1;
                continue;
            }

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

            if let Some(mut section_result) = section_result {
                // Check for leading/trailing whitespace in section name
                let original_name = section_result.display_name.clone();
                let trimmed_name = original_name.trim();
                if trimmed_name != original_name {
                    self.warnings.push(Warning {
                        message: format!(
                            "Section name has leading/trailing whitespace: '{}'. Trimmed to '{}'",
                            original_name, trimmed_name
                        ),
                        file: self.file_path.clone(),
                        line: context.current_line,
                    });
                    // Update to use trimmed name
                    section_result.display_name = trimmed_name.to_string();
                    section_result.id = trimmed_name.to_string();
                }

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
                let names_map = self.section_names_by_parent.entry(parent_id).or_default();

                if let Some(&previous_line) = names_map.get(&section_result.display_name) {
                    // Get parent's display name for error message
                    let parent_name = if let Some(pid) = parent_id {
                        if pid == start_id {
                            "<root>".to_string()
                        } else {
                            match &context.database.blocks[pid].block_type {
                                BlockType::Section {
                                    display_name: parent_display,
                                    ..
                                } => parent_display.clone(),
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

                let block = Block::with_line(
                    BlockType::Section {
                        id: section_result.id,
                        display_name: section_result.display_name,
                    },
                    parent_id,
                    level,
                    context.current_line,
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
                // Try to parse as go-to-section-and-back first (before go-to-section)
                let go_to_and_back_result = match self
                    .go_to_section_and_back_parser
                    .parse(content.trim(), &mut context)
                {
                    Ok(result) => result,
                    Err(e) => {
                        self.collect_error_and_skip(e, &mut context);
                        continue;
                    }
                };

                if let Some(go_to_and_back_result) = go_to_and_back_result {
                    // Check for leading/trailing whitespace in goto path
                    let trimmed_path = go_to_and_back_result.path.trim();
                    let final_path = if trimmed_path != go_to_and_back_result.path {
                        self.warnings.push(Warning {
                            message: format!(
                                "Section name has leading/trailing whitespace: '{}'. Trimmed to '{}'",
                                go_to_and_back_result.path, trimmed_path
                            ),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        });
                        trimmed_path.to_string()
                    } else {
                        go_to_and_back_result.path
                    };

                    // This is a go-to-section-and-back command
                    // Find parent block: check if there's a section at this level first
                    let parent_id = if level < self.last_section_at_level.len() {
                        // There's a section at this level, use it as parent
                        Some(self.last_section_at_level[level])
                    } else if level == 0 {
                        // At level 0 with no section, use first block (Start or a Section)
                        self.last_block_at_level.first().copied()
                    } else if level <= self.last_block_at_level.len() {
                        // No section at this level, pop back to find parent
                        while self.last_block_at_level.len() > level {
                            self.last_block_at_level.pop();
                        }
                        self.last_block_at_level.last().copied()
                    } else {
                        self.collect_error_and_skip(
                            ParseError::InvalidIndentation {
                                message: format!("found {} spaces in: {}", level * 2, content),
                                file: self.file_path.clone(),
                                line: context.current_line,
                            },
                            &mut context,
                        );
                        continue;
                    };

                    // Create GoToSectionAndBack block with placeholder target_block_id
                    // This will be resolved in the validation pass
                    let block = Block::with_line(
                        BlockType::GoToSectionAndBack {
                            path: final_path,
                            target_block_id: 0, // Placeholder, will be resolved
                        },
                        parent_id,
                        level,
                        context.current_line,
                    );
                    let block_id = context.database.add_block(block);

                    // Update last block at this level
                    if level >= self.last_block_at_level.len() {
                        self.last_block_at_level.push(block_id);
                    } else {
                        self.last_block_at_level[level] = block_id;
                    }
                } else {
                    // Try to parse as go-to-section
                    let go_to_result = match self
                        .go_to_section_parser
                        .parse(content.trim(), &mut context)
                    {
                        Ok(result) => result,
                        Err(e) => {
                            self.collect_error_and_skip(e, &mut context);
                            continue;
                        }
                    };

                    if let Some(go_to_result) = go_to_result {
                        // Check for leading/trailing whitespace in goto path
                        let trimmed_path = go_to_result.path.trim();
                        let final_path = if trimmed_path != go_to_result.path {
                            self.warnings.push(Warning {
                            message: format!(
                                "Section name has leading/trailing whitespace: '{}'. Trimmed to '{}'",
                                go_to_result.path, trimmed_path
                            ),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        });
                            trimmed_path.to_string()
                        } else {
                            go_to_result.path
                        };

                        // This is a go-to-section command
                        // Find parent block: check if there's a section at this level first
                        let parent_id = if level < self.last_section_at_level.len() {
                            // There's a section at this level, use it as parent
                            Some(self.last_section_at_level[level])
                        } else if level == 0 {
                            // At level 0 with no section, use first block (Start or a Section)
                            self.last_block_at_level.first().copied()
                        } else if level <= self.last_block_at_level.len() {
                            // No section at this level, pop back to find parent
                            while self.last_block_at_level.len() > level {
                                self.last_block_at_level.pop();
                            }
                            self.last_block_at_level.last().copied()
                        } else {
                            self.collect_error_and_skip(
                                ParseError::InvalidIndentation {
                                    message: format!("found {} spaces in: {}", level * 2, content),
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        };

                        // Create GoToSection block with placeholder target_block_id
                        // This will be resolved in the validation pass
                        let block = Block::with_line(
                            BlockType::GoToSection {
                                path: final_path,
                                target_block_id: 0, // Placeholder, will be resolved
                            },
                            parent_id,
                            level,
                            context.current_line,
                        );
                        let block_id = context.database.add_block(block);

                        // Update last block at this level
                        if level >= self.last_block_at_level.len() {
                            self.last_block_at_level.push(block_id);
                        } else {
                            self.last_block_at_level[level] = block_id;
                        }
                    } else {
                        // Parse as regular string
                        let result = self.line_parser.parse(content.trim(), &mut context)?;

                        // Find parent block: check if there's a section at this level first
                        let parent_id = if level < self.last_section_at_level.len() {
                            // There's a section at this level, use it as parent
                            Some(self.last_section_at_level[level])
                        } else if level == 0 {
                            // At level 0 with no section, use first block (Start or a Section)
                            self.last_block_at_level.first().copied()
                        } else if level <= self.last_block_at_level.len() {
                            // No section at this level, pop back to find parent
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
                        let block = Block::with_line(
                            BlockType::String(string_id),
                            parent_id,
                            level,
                            context.current_line,
                        );
                        let block_id = context.database.add_block(block);

                        // Update last block at this level
                        if level >= self.last_block_at_level.len() {
                            self.last_block_at_level.push(block_id);
                        } else {
                            self.last_block_at_level[level] = block_id;
                        }
                    }
                }
            }

            // Increment line counter after processing each non-empty line
            context.current_line += 1;
        }

        // Add End block (with no parent)
        let end_block = Block::new(BlockType::End, None, 0);
        context.database.add_block(end_block);

        // Run compile-time validation pass
        self.validate_and_resolve(&mut context)?;

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

        Ok((context.database, self.warnings.clone()))
    }

    /// Returns true if the line is a comment (starts with // after optional whitespace).
    ///
    /// Comments can appear at any indentation level and are completely ignored by the parser.
    /// Only line-level comments are supported; inline comments (// after content) are not detected.
    fn is_comment(line: &str) -> bool {
        line.trim_start().starts_with("//")
    }

    fn parse_indentation<'a>(
        &self,
        line: &'a str,
        context: &ParserContext,
    ) -> Result<(usize, &'a str), ParseError> {
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

    /// Validate and resolve all GoToSection blocks
    /// This runs after parsing and performs compile-time checks
    fn validate_and_resolve(&mut self, context: &mut ParserContext) -> Result<(), ParseError> {
        // Build section registry: map section paths to (BlockId, line_number)
        let section_registry = self.build_section_registry(&context.database);

        // Validate section names don't contain backslash
        self.validate_section_names(&context.database)?;

        // Only detect empty sections if there were no parse errors
        // (parse errors may have prevented blocks from being created)
        if self.errors.is_empty() {
            self.detect_empty_sections(&context.database)?;
        }

        // Collect GoToSection and GoToSectionAndBack blocks first to avoid borrow checker issues
        let goto_blocks: Vec<(BlockId, String, usize, bool)> = context
            .database
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(block_id, block)| match &block.block_type {
                BlockType::GoToSection { path, .. } => {
                    Some((block_id, path.clone(), block.line, false))
                }
                BlockType::GoToSectionAndBack { path, .. } => {
                    Some((block_id, path.clone(), block.line, true))
                }
                _ => None,
            })
            .collect();

        // Resolve and validate all GoToSection and GoToSectionAndBack blocks
        for (block_id, path, line, is_call_and_back) in goto_blocks {
            // Find the containing section for this block
            let containing_section = self.find_containing_section(&context.database, block_id);

            // Resolve the path
            match self.resolve_path(
                &path,
                containing_section,
                &section_registry,
                &context.database,
                line,
            ) {
                Ok(target_block_id) => {
                    // Update the block with the resolved target
                    if is_call_and_back {
                        context.database.blocks[block_id].block_type =
                            BlockType::GoToSectionAndBack {
                                path: path.clone(),
                                target_block_id,
                            };

                        // Warn if calling END (it will not return)
                        if path.trim() == "END" {
                            self.warnings.push(Warning {
                                message: "<-> END will not return (just end execution)".to_string(),
                                file: self.file_path.clone(),
                                line,
                            });
                        }
                    } else {
                        context.database.blocks[block_id].block_type = BlockType::GoToSection {
                            path: path.clone(),
                            target_block_id,
                        };
                    }
                }
                Err(e) => {
                    // Collect the error
                    self.errors.push(e);
                }
            }

            // Check for unreachable code after this block
            // Only for GoToSection (->), not GoToSectionAndBack (<->)
            if !is_call_and_back {
                self.detect_unreachable_code(&context.database, block_id);
            }
        }

        Ok(())
    }

    /// Build a registry mapping section paths to BlockIds
    fn build_section_registry(&self, database: &Database) -> HashMap<String, (BlockId, usize)> {
        let mut registry = HashMap::new();

        for (block_id, block) in database.blocks.iter().enumerate() {
            if let BlockType::Section { .. } = &block.block_type {
                // Build the full path for this section
                let path = self.build_section_path_string(database, block_id);
                // Line number is block_id for simplicity (we don't track actual line numbers per block yet)
                registry.insert(path, (block_id, block_id));
            }
        }

        registry
    }

    /// Build the full hierarchical path string for a section block
    ///
    /// Walks up the parent chain to construct a path like "Root \ Child \ Grandchild".
    /// Used for building the section registry and error messages.
    /// Returns an empty string if the block is not a section.
    fn build_section_path_string(&self, database: &Database, block_id: BlockId) -> String {
        let mut path_parts = Vec::new();
        let mut current_id = block_id;

        // Walk up the parent chain, collecting section names
        while let Some(parent_id) = database.blocks[current_id].parent_id {
            if let BlockType::Section { display_name, .. } = &database.blocks[current_id].block_type
            {
                path_parts.push(display_name.clone());
            }
            current_id = parent_id;
        }

        // Reverse to get top-down order
        path_parts.reverse();
        path_parts.join(" \\ ")
    }

    /// Find the nearest ancestor section that contains this block
    ///
    /// Walks up the parent chain until a Section block is found.
    /// Returns None if the block is at the root level (no containing section).
    fn find_containing_section(&self, database: &Database, block_id: BlockId) -> Option<BlockId> {
        let mut current_id = block_id;

        // Walk up parents until we find a Section block
        while let Some(parent_id) = database.blocks[current_id].parent_id {
            if matches!(
                database.blocks[parent_id].block_type,
                BlockType::Section { .. }
            ) {
                return Some(parent_id);
            }
            current_id = parent_id;
        }

        None
    }

    /// Resolve a go-to-section path to a target BlockId
    ///
    /// This function handles several path formats:
    /// - Absolute paths: `"Root \ Child"` - searches from the root
    /// - Relative paths: `"Child"` or `"Sibling"` - searches children first, then siblings
    /// - Parent navigation: `".."` - navigates to containing section's parent
    /// - Current section: `"."` - refers to the containing section itself
    /// - Multi-level parent: `".. \ .."` - navigates up multiple levels
    /// - Combined: `".. \ Sibling"` - goes to parent, then to sibling
    ///
    /// Search order for relative paths:
    /// 1. Children of containing section
    /// 2. Siblings of containing section
    /// 3. Check full registry for absolute match
    ///
    /// Returns the BlockId of the target section, or an error if not found
    fn resolve_path(
        &mut self,
        path: &str,
        containing_section: Option<BlockId>,
        registry: &HashMap<String, (BlockId, usize)>,
        database: &Database,
        line: usize,
    ) -> Result<BlockId, ParseError> {
        let path = path.trim();

        // Handle "END" (jump to end)
        if path == "END" {
            // END block is always the last block in the database
            return Ok(database.blocks.len() - 1);
        }

        // Handle "." (current section)
        if path == "." {
            if let Some(section_id) = containing_section {
                return Ok(section_id);
            } else {
                return Err(ParseError::SectionNotFound {
                    path: path.to_string(),
                    file: self.file_path.clone(),
                    line,
                });
            }
        }

        // Parse the path into segments
        let segments: Vec<&str> = path.split(" \\ ").map(|s| s.trim()).collect();

        // Check if this is an absolute path (doesn't start with ..)
        if !segments[0].starts_with("..") {
            // Try absolute path first
            if let Some(&(block_id, _)) = registry.get(path) {
                return Ok(block_id);
            }

            // Try relative path (search children and siblings)
            if let Some(section_id) = containing_section {
                // Search children first
                if let Some(child_id) = self.find_child_section(database, section_id, segments[0]) {
                    if segments.len() == 1 {
                        return Ok(child_id);
                    }
                    // For longer paths, build the full path and look it up
                    let full_path = segments.join(" \\ ");
                    if let Some(&(block_id, _)) = registry.get(&full_path) {
                        return Ok(block_id);
                    }
                }

                // Search siblings
                if let Some(sibling_id) =
                    self.find_sibling_section(database, section_id, segments[0])
                {
                    if segments.len() == 1 {
                        return Ok(sibling_id);
                    }
                    // For longer paths, build the full path and look it up
                    let full_path = segments.join(" \\ ");
                    if let Some(&(block_id, _)) = registry.get(&full_path) {
                        return Ok(block_id);
                    }
                }
            }
        } else {
            // Handle ".." navigation
            let mut current_section = containing_section;
            let mut segment_index = 0;

            // Process ".." segments
            while segment_index < segments.len() && segments[segment_index] == ".." {
                if let Some(section_id) = current_section {
                    // Navigate to parent section
                    current_section = self.find_parent_section(database, section_id);
                    if current_section.is_none() {
                        return Err(ParseError::NavigationAboveRoot {
                            file: self.file_path.clone(),
                            line,
                        });
                    }
                } else {
                    return Err(ParseError::NavigationAboveRoot {
                        file: self.file_path.clone(),
                        line,
                    });
                }
                segment_index += 1;
            }

            // If there are more segments, resolve them
            if segment_index < segments.len() {
                if let Some(section_id) = current_section {
                    // Look for the rest of the path as siblings
                    let remaining_path = segments[segment_index..].join(" \\ ");
                    if let Some(sibling_id) =
                        self.find_sibling_section(database, section_id, &remaining_path)
                    {
                        return Ok(sibling_id);
                    }
                    // Try building the full path from current section
                    let current_path = self.build_section_path_string(database, section_id);
                    if let Some(parent_id) = database.blocks[section_id].parent_id {
                        let full_path = if current_path.is_empty() {
                            remaining_path.clone()
                        } else {
                            format!(
                                "{} \\ {}",
                                self.build_section_path_string(database, parent_id),
                                remaining_path
                            )
                        };
                        if let Some(&(block_id, _)) = registry.get(&full_path) {
                            return Ok(block_id);
                        }
                    }
                }
            } else {
                // Just "..", return the parent section
                if let Some(section_id) = current_section {
                    return Ok(section_id);
                }
            }
        }

        Err(ParseError::SectionNotFound {
            path: path.to_string(),
            file: self.file_path.clone(),
            line,
        })
    }

    /// Find a direct child section of the given parent by display name
    ///
    /// Searches only immediate children, not nested descendants.
    /// Returns the BlockId of the matching section, or None if not found.
    fn find_child_section(
        &self,
        database: &Database,
        parent_id: BlockId,
        name: &str,
    ) -> Option<BlockId> {
        for &child_id in &database.blocks[parent_id].children {
            if let BlockType::Section { display_name, .. } = &database.blocks[child_id].block_type {
                if display_name == name {
                    return Some(child_id);
                }
            }
        }
        None
    }

    /// Find a sibling section (shares same parent) by display name
    ///
    /// Searches all children of the given section's parent, excluding the section itself.
    /// Returns the BlockId of the matching sibling, or None if not found or if section has no parent.
    fn find_sibling_section(
        &self,
        database: &Database,
        section_id: BlockId,
        name: &str,
    ) -> Option<BlockId> {
        if let Some(parent_id) = database.blocks[section_id].parent_id {
            for &sibling_id in &database.blocks[parent_id].children {
                if sibling_id != section_id {
                    if let BlockType::Section { display_name, .. } =
                        &database.blocks[sibling_id].block_type
                    {
                        if display_name == name {
                            return Some(sibling_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Find the parent section of a given section
    ///
    /// Walks up the parent chain to find the nearest ancestor that is also a Section block.
    /// Used for resolving ".." navigation. Returns None if the section is at the root.
    fn find_parent_section(&self, database: &Database, section_id: BlockId) -> Option<BlockId> {
        let mut current_id = section_id;

        while let Some(parent_id) = database.blocks[current_id].parent_id {
            if matches!(
                database.blocks[parent_id].block_type,
                BlockType::Section { .. }
            ) {
                return Some(parent_id);
            }
            current_id = parent_id;
        }

        None
    }

    /// Validate that section names don't contain backslash and aren't reserved words
    fn validate_section_names(&mut self, database: &Database) -> Result<(), ParseError> {
        for block in database.blocks.iter() {
            if let BlockType::Section { display_name, .. } = &block.block_type {
                // Check for reserved word "END"
                if display_name == "END" {
                    self.errors.push(ParseError::InvalidSectionName {
                        message: "Section name \"END\" is reserved".to_string(),
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }

                // Check for backslash
                if display_name.contains('\\') {
                    self.errors.push(ParseError::InvalidSectionName {
                        message: "Section names cannot contain '\\' character".to_string(),
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }
            }
        }
        Ok(())
    }

    /// Detect empty sections
    fn detect_empty_sections(&mut self, database: &Database) -> Result<(), ParseError> {
        for (block_id, block) in database.blocks.iter().enumerate() {
            if let BlockType::Section { display_name, .. } = &block.block_type {
                // Check if this section has any non-section children (recursively)
                let has_content = Self::section_has_content(database, block_id);

                if !has_content {
                    self.errors.push(ParseError::EmptySection {
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }
            }
        }
        Ok(())
    }

    /// Check if a section has any content (String or GoToSection blocks), recursively
    fn section_has_content(database: &Database, section_id: BlockId) -> bool {
        for &child_id in &database.blocks[section_id].children {
            match &database.blocks[child_id].block_type {
                BlockType::String(_) | BlockType::GoToSection { .. } => return true,
                BlockType::Section { .. } => {
                    // Recursively check subsections
                    if Self::section_has_content(database, child_id) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Detect unreachable code after a GoToSection block
    fn detect_unreachable_code(&mut self, database: &Database, goto_block_id: BlockId) {
        let goto_block = &database.blocks[goto_block_id];

        // Check for sibling blocks after this one
        if let Some(parent_id) = goto_block.parent_id {
            let parent = &database.blocks[parent_id];
            if let Some(pos) = parent.children.iter().position(|&id| id == goto_block_id) {
                // Siblings after this one are unreachable UNLESS they are sections
                // (sections can be jumped to, so they're not unreachable)
                for &sibling_id in &parent.children[pos + 1..] {
                    let sibling = &database.blocks[sibling_id];
                    // Only warn about non-section blocks
                    if !matches!(sibling.block_type, BlockType::Section { .. }) {
                        self.warnings.push(Warning {
                            message: "Unreachable code after section jump".to_string(),
                            file: self.file_path.clone(),
                            line: sibling.line,
                        });
                    }
                }
            }
        }

        // Check for child blocks (they are also unreachable)
        // BUT don't warn about section children - they can be jumped to
        for &child_id in &goto_block.children {
            let child = &database.blocks[child_id];
            if !matches!(child.block_type, BlockType::Section { .. }) {
                self.warnings.push(Warning {
                    message: "Unreachable code after section jump".to_string(),
                    file: self.file_path.clone(),
                    line: child.line,
                });
            }
        }
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
        let (database, _warnings) = parser.parse(test_case.script).unwrap();

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
        let (database, _warnings) = parser.parse(script).unwrap();

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
    fn test_warning_section_whitespace() {
        // Note: "# " takes one space, so "#  " leaves one extra leading space
        // Trailing spaces are preserved in the string literal
        let script = "#  Section A  \nText in A";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have 1 warning for the section name
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0]
            .message
            .contains("Section name has leading/trailing whitespace"));
        assert!(warnings[0].message.contains("Section A"));
        assert_eq!(warnings[0].line, 1);
    }

    #[test]
    fn test_warning_unreachable_siblings() {
        let script = "# Section A\nText before\n-> Section B\nText after sibling 1\nText after sibling 2\n\n# Section B\nText in B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have 2 warnings for unreachable siblings
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0]
            .message
            .contains("Unreachable code after section jump"));
        assert!(warnings[1]
            .message
            .contains("Unreachable code after section jump"));
        assert_eq!(warnings[0].line, 4); // "Text after sibling 1"
        assert_eq!(warnings[1].line, 5); // "Text after sibling 2"
    }

    #[test]
    fn test_warning_unreachable_children() {
        let script = "# Section A\nText before\n-> Section B\n  Text child 1\n  Text child 2\n\n# Section B\nText in B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have 2 warnings for unreachable children
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0]
            .message
            .contains("Unreachable code after section jump"));
        assert!(warnings[1]
            .message
            .contains("Unreachable code after section jump"));
        assert_eq!(warnings[0].line, 4); // "  Text child 1"
        assert_eq!(warnings[1].line, 5); // "  Text child 2"
    }

    #[test]
    fn test_subsections_not_unreachable() {
        // Subsections should NOT be flagged as unreachable even if there's a goto in the parent
        let script = "# Parent\n-> Child A\n  ## Child A\n  Text in child A\n  ## Child B\n  Text in child B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have NO warnings - subsections are not unreachable
        assert_eq!(
            warnings.len(),
            0,
            "Subsections should not be flagged as unreachable. Got warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn test_error_section_not_found() {
        let script = "# Section A\n-> Nonexistent Section";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::SectionNotFound { path, line, .. }) => {
                assert_eq!(path, "Nonexistent Section");
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::SectionNotFound { path, line, .. }
                    if path == "Nonexistent Section" && *line == 2)
                ));
            }
            _ => panic!("Expected SectionNotFound error"),
        }
    }

    #[test]
    fn test_error_navigation_above_root() {
        let script = "# Section A\n-> ..";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::NavigationAboveRoot { line, .. }) => {
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors
                    .iter()
                    .any(|e| matches!(e, ParseError::NavigationAboveRoot { line, .. }
                    if *line == 2)));
            }
            _ => panic!("Expected NavigationAboveRoot error"),
        }
    }

    #[test]
    fn test_error_empty_section() {
        // An empty section has no children at all (not even subsections or goto commands)
        let script = "# Section A\n\n# Section B\nText in B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::EmptySection { name, line, .. }) => {
                assert_eq!(name, "Section A");
                assert_eq!(line, 1);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                // Could have multiple empty section errors
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::EmptySection { name, .. } if name == "Section A")
                ));
            }
            _ => panic!("Expected EmptySection error"),
        }
    }

    #[test]
    fn test_section_with_only_goto_is_not_empty() {
        // Bug: sections with only goto commands should NOT be flagged as empty
        let script =
            "# Section A\n-> Section C\n\n# Section B\nText in B\n\n# Section C\nText in C";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        // Should succeed - sections with goto commands are valid
        assert!(
            result.is_ok(),
            "Section with only goto command should be valid, got error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_jump_to_parent_with_dotdot() {
        // Test that -> .. correctly resolves to parent section
        let script = "# Parent\nText in parent\n  ## Child\n  Text in child\n  -> ..";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        assert!(
            result.is_ok(),
            "Jump to parent using .. should work, got error: {:?}",
            result.err()
        );

        let (database, _warnings) = result.unwrap();

        // Find the GoToSection block
        let goto_block = database
            .blocks
            .iter()
            .find(|b| matches!(b.block_type, BlockType::GoToSection { .. }));
        assert!(goto_block.is_some(), "Should have a GoToSection block");

        // Check that it resolves to the Parent section (block 1)
        if let Some(block) = goto_block {
            if let BlockType::GoToSection {
                target_block_id, ..
            } = block.block_type
            {
                assert_eq!(
                    target_block_id, 1,
                    "Should resolve to Parent section (block 1)"
                );
            }
        }
    }

    #[test]
    fn test_error_malformed_empty_reference() {
        let script = "# Section A\n->";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section name after '->'"));
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section name after '->'") && *line == 2)
                ));
            }
            Err(e) => panic!("Expected InvalidGoToSection error, got: {:?}", e),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_error_malformed_trailing_backslash() {
        let script = "# Section A\n-> Root \\";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section name after '->'"));
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section name after '->'") && *line == 2)
                ));
            }
            _ => panic!("Expected InvalidGoToSection error"),
        }
    }

    #[test]
    fn test_error_malformed_no_space_after_arrow() {
        let script = "# Section A\n->Section A";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section name after '->'"));
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section name after '->'") && *line == 2)
                ));
            }
            _ => panic!("Expected InvalidGoToSection error"),
        }
    }

    #[test]
    fn test_warning_extra_spaces_in_goto_path() {
        // Extra spaces in goto path should be accepted with a warning (not an error)
        let script = "# Section A\nText\n->  Section A";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        // Should succeed (not error)
        assert!(
            result.is_ok(),
            "Extra spaces in goto path should be accepted, got error: {:?}",
            result.err()
        );

        let (_database, warnings) = result.unwrap();
        // Should have a warning about the extra space
        assert!(warnings.len() > 0, "Should have warning about whitespace");
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("whitespace") || w.message.contains("space")),
            "Should warn about whitespace, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_error_malformed_wrong_spacing_around_backslash() {
        let script = "# Root\n  ## Child\nText\n-> Root\\Child";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section name after '->'"));
                assert_eq!(line, 4);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section name after '->'") && *line == 4)
                ));
            }
            _ => panic!("Expected InvalidGoToSection error"),
        }
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
        let (database, _warnings) = parser.parse(script).unwrap();

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
    fn test_is_comment() {
        assert!(Parser::is_comment("// This is a comment"));
        assert!(Parser::is_comment("  // Indented comment"));
        assert!(Parser::is_comment("    // More indentation"));
        assert!(Parser::is_comment("//"));
        assert!(Parser::is_comment("/// Triple slash"));
        assert!(Parser::is_comment("//// Quadruple slash"));
        assert!(!Parser::is_comment("Not a comment"));
        assert!(!Parser::is_comment("# Section"));
        assert!(!Parser::is_comment("Text with // in middle"));
    }

    #[test]
    fn test_comment_parsing() {
        let script =
            "// Comment at start\nFirst line\n// Comment in middle\nSecond line\n// Comment at end";
        let mut parser = Parser::new();
        let (database, _warnings) = parser.parse(script).unwrap();

        // Should have Start, two string blocks, and End
        assert_eq!(database.blocks.len(), 4);
        assert_eq!(database.strings.len(), 2);
        assert_eq!(database.strings[0], "First line");
        assert_eq!(database.strings[1], "Second line");
    }

    #[test]
    fn test_comment_with_arbitrary_indentation() {
        let script = "First line\n      // Comment with weird indentation\nSecond line";
        let mut parser = Parser::new();
        let (database, _warnings) = parser.parse(script).unwrap();

        // Should have Start, two string blocks, and End (comment ignored)
        assert_eq!(database.blocks.len(), 4);
        assert_eq!(database.strings.len(), 2);
        assert_eq!(database.strings[0], "First line");
        assert_eq!(database.strings[1], "Second line");
    }
}
