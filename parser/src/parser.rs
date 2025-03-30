use cuentitos_common::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::parsers::{FeatureParser, ParserContext};
use crate::parsers::line_parser::LineParser;
use crate::parsers::section_parser::SectionParser;
use crate::{ParseError, ParseErrors};

pub struct Parser {
    last_block_at_level: Vec<BlockId>,
    last_section_at_level: Vec<BlockId>,  // Track the last section block at each level
    current_line: usize,
    file_path: PathBuf,
    section_map: HashMap<(String, usize), (BlockId, usize)>, // (section_name, level) -> (block_id, line_number)
    line_parser: LineParser,
    section_parser: SectionParser,
    in_section_hierarchy: bool, // Track whether we're in a section hierarchy
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
            in_section_hierarchy: false,
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
            in_section_hierarchy: false,
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

    pub fn parse(&mut self, script: &str) -> Result<Database, ParseErrors> {
        self.current_line = 0;
        let mut db = Database::new();
        let mut context = ParserContext::new();
        context.file_path = Some(self.file_path.clone());

        // Add START block
        let start_id = db.add_block(Block::new(BlockType::Start, None, 0));
        dbg!("Created START block", start_id);
        self.last_block_at_level.push(start_id);
        self.last_section_at_level.push(start_id);
        dbg!("Initial tracking arrays", &self.last_block_at_level, &self.last_section_at_level);

        // Track all errors encountered during parsing
        let mut errors = Vec::new();

        for line in script.lines() {
            self.current_line += 1;
            context.current_line = self.current_line;

            if line.trim().is_empty() {
                continue;
            }

            let (level, content) = match self.parse_indentation(line) {
                Ok(result) => result,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };
            dbg!(self.current_line, &line, level, &content);
            dbg!("Current tracking state", &self.last_block_at_level, &self.last_section_at_level);
            dbg!("Current hierarchy mode", self.in_section_hierarchy);

            if content.starts_with('#') {
                // When we see a section, we're definitely in section hierarchy mode
                self.in_section_hierarchy = true;
                dbg!("Entering section hierarchy mode");

                if let Some(section_result) = match self.section_parser.parse(content, &mut context) {
                    Ok(result) => result,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                } {
                    dbg!("Parsing section", &section_result.title, section_result.level);
                    dbg!("Section tracking before", &self.last_section_at_level);

                    // Check for orphaned sub-sections
                    if section_result.level > 0 {
                        // For a sub-section at level N, we need a section at level N-1
                        let parent_level = section_result.level - 1;
                        dbg!("Checking parent level", parent_level);

                        // Ensure we have enough levels tracked
                        while self.last_section_at_level.len() <= parent_level {
                            dbg!("Extending section tracking to level", self.last_section_at_level.len());
                            self.last_section_at_level.push(start_id);
                        }

                        let parent_id = self.last_section_at_level[parent_level];
                        dbg!("Found parent section", parent_id, &db.blocks[parent_id].block_type);
                        if !matches!(db.blocks[parent_id].block_type, BlockType::Section(_)) {
                            dbg!("No valid parent section found", &section_result.title, section_result.level, &self.last_section_at_level);
                            errors.push(ParseError::OrphanedSubSection {
                                file: self.file_path.clone(),
                                line: self.current_line,
                            });
                            continue;
                        }
                    }

                    // Get parent section name if this is a sub-section
                    let parent_name = if section_result.level > 0 {
                        let parent_id = self.last_section_at_level[section_result.level - 1];
                        dbg!("Parent section lookup", section_result.level - 1, parent_id);
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
                        errors.push(ParseError::DuplicateSectionName {
                            file: self.file_path.clone(),
                            line: self.current_line,
                            name: section_result.title.clone(),
                            parent: parent_name,
                            previous_line: *previous_line,
                        });
                        continue;
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
                        dbg!("Extending section tracking array to level", self.last_section_at_level.len());
                        self.last_section_at_level.push(block_id);
                    }
                    self.last_section_at_level[section_result.level] = block_id;
                    dbg!("Updated last_section_at_level", &self.last_section_at_level);

                    // Also update last_block_at_level
                    while self.last_block_at_level.len() <= section_result.level {
                        dbg!("Extending block tracking array to level", self.last_block_at_level.len());
                        self.last_block_at_level.push(block_id);
                    }
                    self.last_block_at_level[section_result.level] = block_id;
                    dbg!("Updated last_block_at_level", &self.last_block_at_level);
                }
            } else {
                let line_result = match self.line_parser.parse(content, &mut context) {
                    Ok(result) => result,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                dbg!("Parsing text block", &line_result.string, level);
                let string_id = db.add_string(line_result.string);
                dbg!("Text block tracking state", &self.last_block_at_level, &self.last_section_at_level);

                // Determine parent based on whether we're in a section hierarchy
                let parent = if self.in_section_hierarchy {
                    // In section hierarchy, parent to the section at this level
                    while self.last_section_at_level.len() <= level {
                        dbg!("Extending section tracking array", self.last_section_at_level.len());
                        self.last_section_at_level.push(self.last_section_at_level[0]);
                    }
                    let parent = self.last_section_at_level[level];
                    dbg!("Found section parent at same level", level, parent, &db.blocks[parent].block_type);
                    parent
                } else {
                    // In text block hierarchy, parent to the last block at previous level
                    if level == 0 {
                        dbg!("Root level text block in text hierarchy", start_id);
                        start_id
                    } else {
                        dbg!("Looking for text block parent at level", level - 1);
                        let parent = self.last_block_at_level[level - 1];
                        dbg!("Found text block parent", parent, &db.blocks[parent].block_type);
                        parent
                    }
                };

                let block = Block::new(BlockType::String(string_id), Some(parent), level);
                let block_id = db.add_block(block);
                dbg!("Added text block", block_id, parent, level);
                dbg!("Block parent relationship", block_id, "->", parent);

                // Update last_block_at_level for the current level
                while self.last_block_at_level.len() <= level {
                    dbg!("Extending block tracking array", self.last_block_at_level.len());
                    self.last_block_at_level.push(block_id);
                }
                self.last_block_at_level[level] = block_id;
                dbg!("Updated last_block_at_level", &self.last_block_at_level);
            }
        }

        // Add END block
        let end_id = db.add_block(Block::new(BlockType::End, Some(start_id), 0));
        dbg!("Added END block", end_id);

        // If we encountered any errors, return them all together
        if !errors.is_empty() {
            return Err(ParseErrors(errors));
        }

        Ok(db)
    }
}
