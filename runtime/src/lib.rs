use cuentitos_common::*;

/// The runtime engine that executes a cuentitos script.
///
/// The Runtime is responsible for:
/// - Managing the execution state of the script
/// - Traversing blocks in the correct order
/// - Maintaining section hierarchy
/// - Providing access to the current execution state
pub struct Runtime {
    /// The database containing all blocks and strings
    pub database: Database,
    /// Whether the runtime is currently executing
    running: bool,
    /// The index of the current block being executed
    program_counter: usize,
    /// The index of the previously executed block
    previous_program_counter: usize,
    /// Track the current execution path through block IDs
    current_path: Vec<BlockId>,
}

impl Runtime {
    /// Creates a new Runtime instance with the given database.
    ///
    /// # Arguments
    ///
    /// * `database` - The database containing blocks and strings to execute
    pub fn new(database: Database) -> Self {
        Self {
            database,
            running: false,
            program_counter: 0,
            previous_program_counter: 0,
            current_path: Vec::new(),
        }
    }

    /// Starts the runtime execution from the beginning.
    ///
    /// This will:
    /// - Set the running state to true
    /// - Reset the program counter to 0
    /// - Clear the current path
    /// - Initialize with the START block if available
    pub fn run(&mut self) {
        self.running = true;
        self.program_counter = 0;
        self.current_path.clear();
        if !self.database.blocks.is_empty() {
            self.current_path.push(0); // Start block
        }
    }

    /// Stops the runtime execution.
    ///
    /// This will:
    /// - Set the running state to false
    /// - Reset the program counter to 0
    /// - Clear the current path
    pub fn stop(&mut self) {
        self.running = false;
        self.program_counter = 0;
        self.current_path.clear();
    }

    /// Returns whether the runtime is currently executing.
    pub fn running(&self) -> bool {
        self.running
    }

    /// Returns whether the runtime can continue executing.
    ///
    /// This is true when:
    /// - The runtime is running
    /// - The END block hasn't been reached
    /// - There are blocks in the database
    pub fn can_continue(&self) -> bool {
        self.running && !self.has_ended() && !self.database.blocks.is_empty()
    }

    /// Returns whether the runtime has reached the END block.
    pub fn has_ended(&self) -> bool {
        matches!(
            self.current_block().map(|b| b.block_type),
            Some(BlockType::End)
        )
    }

    /// Returns all blocks that should be displayed at the current execution point.
    ///
    /// This includes:
    /// - All blocks up to the current program counter
    /// - All relevant section headers for the current context
    /// - The START and END blocks when appropriate
    pub fn current_blocks(&self) -> Vec<Block> {
        if !self.running || self.database.blocks.is_empty() {
            Vec::new()
        } else if self.has_ended() {
            // When we've reached the end, show all blocks up to and including END
            let mut blocks = Vec::new();
            let mut current_sections: Vec<Block> = Vec::new();

            for block in &self.database.blocks {
                match block.block_type {
                    BlockType::Start | BlockType::End => blocks.push(block.clone()),
                    BlockType::String(_) => {
                        // Show all current sections before each text block
                        for section in &current_sections {
                            blocks.push(section.clone());
                        }
                        blocks.push(block.clone());
                    }
                    BlockType::Section(_) => {
                        // Update current sections based on level
                        while let Some(last) = current_sections.last() {
                            if last.level >= block.level {
                                current_sections.pop();
                            } else {
                                break;
                            }
                        }
                        current_sections.push(block.clone());
                    }
                }
            }
            blocks
        } else {
            // Otherwise show blocks up to current position
            let mut blocks = Vec::new();
            let mut current_sections: Vec<Block> = Vec::new();

            for block in &self.database.blocks[0..=self.program_counter] {
                match block.block_type {
                    BlockType::Start | BlockType::End => blocks.push(block.clone()),
                    BlockType::String(_) => {
                        // Show all current sections before each text block
                        for section in &current_sections {
                            blocks.push(section.clone());
                        }
                        blocks.push(block.clone());
                    }
                    BlockType::Section(_) => {
                        // Update current sections based on level
                        while let Some(last) = current_sections.last() {
                            if last.level >= block.level {
                                current_sections.pop();
                            } else {
                                break;
                            }
                        }
                        current_sections.push(block.clone());
                    }
                }
            }
            blocks
        }
    }

    /// Returns the current block being executed, if any.
    ///
    /// # Returns
    ///
    /// * `Some(Block)` - The current block if the runtime is running and has blocks
    /// * `None` - If the runtime is not running or has no blocks
    pub fn current_block(&self) -> Option<Block> {
        if self.running() && !self.database.blocks.is_empty() {
            if self.program_counter >= self.database.blocks.len() {
                None
            } else {
                Some(self.database.blocks[self.program_counter].clone())
            }
        } else {
            None
        }
    }

    // Find the next block to visit in a depth-first traversal
    fn find_next_block(&self) -> Option<usize> {
        if self.program_counter >= self.database.blocks.len() - 1 {
            return None;
        }

        let current_block = &self.database.blocks[self.program_counter];

        // First, try to find a child
        if !current_block.children.is_empty() {
            return Some(current_block.children[0]);
        }

        // If no children, try to find the next sibling
        let mut current_id = self.program_counter;
        while let Some(parent_id) = self.database.blocks[current_id].parent_id {
            let parent = &self.database.blocks[parent_id];
            let current_index = parent
                .children
                .iter()
                .position(|&id| id == current_id)
                .unwrap();

            // If there's a next sibling, return it
            if current_index + 1 < parent.children.len() {
                return Some(parent.children[current_index + 1]);
            }

            // Otherwise, move up to the parent and try again
            current_id = parent_id;
        }

        // For disconnected blocks or when we've exhausted all siblings and parents,
        // just move to the next sequential block if it exists
        if self.program_counter + 1 < self.database.blocks.len() {
            Some(self.program_counter + 1)
        } else {
            None
        }
    }

    /// Advances the runtime to the next block in the execution sequence.
    ///
    /// # Returns
    ///
    /// `true` if successfully moved to the next block, `false` if no more blocks
    /// are available or the runtime cannot continue.
    pub fn step(&mut self) -> bool {
        if self.can_continue() {
            if let Some(next_id) = self.find_next_block() {
                self.previous_program_counter = self.program_counter;
                self.program_counter = next_id;
                self.current_path.push(next_id);
                return true;
            }
        }
        false
    }

    /// Skips to the end of the script.
    ///
    /// This will:
    /// - Continue stepping until the END block is reached
    /// - Update the previous program counter to show all skipped blocks
    ///
    /// # Returns
    ///
    /// `true` if successfully skipped to the end, `false` otherwise.
    pub fn skip(&mut self) -> bool {
        let previous_program_counter = self.program_counter;

        // Keep stepping until we reach the END block
        while !self.has_ended() && self.can_continue() {
            self.step();
        }

        // Set previous_program_counter to show all blocks that were skipped
        if self.program_counter > previous_program_counter {
            self.previous_program_counter = previous_program_counter;
        }

        true
    }

    /// Returns the current section hierarchy, from root to leaf.
    ///
    /// For example, if we're in "Chapter 1 > Section 2 > Subsection 3",
    /// this will return [Chapter 1, Section 2, Subsection 3] in that order.
    ///
    /// # Returns
    ///
    /// A vector of Block objects representing the current section hierarchy,
    /// ordered from root (outermost) to leaf (innermost).
    pub fn current_section_hierarchy(&self) -> Vec<Block> {
        let mut sections = Vec::new();
        let current_id = self.program_counter;

        // Get the current block
        let current_block = match self.database.blocks.get(current_id) {
            Some(block) => block,
            None => return sections,
        };

        // If we're at a section block, include it
        if matches!(current_block.block_type, BlockType::Section(_)) {
            sections.push(current_block.clone());
        }

        // Start from the parent of the current block
        let start_id = current_block.parent_id;

        // Walk up the parent chain until we reach the root
        let mut current_id = match start_id {
            Some(id) => id,
            None => return sections,
        };

        while let Some(block) = self.database.blocks.get(current_id) {
            // If this is a section block, check if we should include it
            if matches!(block.block_type, BlockType::Section(_)) {
                // Only include sections that are at a lower level than the current section
                let should_include = if let Some(current_section) = sections.last() {
                    block.level < current_section.level
                } else {
                    // If no sections yet, always include
                    true
                };

                if should_include {
                    sections.push(block.clone());
                }
            }

            // Move up to the parent
            if let Some(parent_id) = block.parent_id {
                current_id = parent_id;
            } else {
                break;
            }
        }

        // Reverse to get root-to-leaf order
        sections.reverse();

        sections
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cuentitos_common::test_case::TestCase;

    #[test]
    fn accepts_database() {
        let database = cuentitos_common::Database::default();
        let runtime = Runtime::new(database.clone());
        assert_eq!(runtime.database, database);
    }

    #[test]
    fn run_initiates_runtime() {
        let database = cuentitos_common::Database::default();
        let mut runtime = Runtime::new(database.clone());

        assert_eq!(runtime.running(), false);

        runtime.run();

        assert_eq!(runtime.running(), true);
    }

    #[test]
    fn get_current_block() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        assert_eq!(runtime.current_block(), None);

        runtime.run();

        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::Start)
        ));
        assert_eq!(runtime.current_blocks().len(), 1);

        runtime.step();

        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::String(id) => {
                    assert_eq!(runtime.database.strings[id], "This is a single line")
                }
                _ => panic!("Expected String block"),
            }
        } else {
            panic!("Expected block to be present");
        }

        runtime.step();

        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::String(id) => {
                    assert_eq!(runtime.database.strings[id], "This is another line of text")
                }
                _ => panic!("Expected String block"),
            }
        } else {
            panic!("Expected block to be present");
        }

        runtime.step();

        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::End)
        ));
    }

    #[test]
    fn step_moves_to_next_line() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        runtime.run();

        runtime.step();

        runtime.step();

        assert_eq!(runtime.program_counter, 2);

        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::String(id) => {
                    assert_eq!(runtime.database.strings[id], "This is another line of text")
                }
                _ => panic!("Expected String block"),
            }
        } else {
            panic!("Expected block to be present");
        }
    }

    #[test]
    fn skip_moves_to_end() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000003-two-lines-and-skip.md"),
            "00000000003-two-lines-and-skip.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        runtime.run();

        runtime.skip();

        assert_eq!(runtime.can_continue(), false);

        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::End)
        ));
    }

    #[test]
    fn skip_and_current_blocks_show_intermediate_blocks() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000003-two-lines-and-skip.md"),
            "00000000003-two-lines-and-skip.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        runtime.skip();

        assert_eq!(runtime.can_continue(), false);

        // After skip, current_blocks shows all blocks that were skipped
        let blocks = runtime.current_blocks();

        // We expect: Start, String1, String2, End
        assert_eq!(blocks.len(), 4);

        // Start block
        assert!(matches!(blocks[0].block_type, BlockType::Start));

        // First string
        match &blocks[1].block_type {
            BlockType::String(id) => {
                assert_eq!(runtime.database.strings[*id], "This is a single line")
            }
            _ => panic!("Expected String block"),
        }

        // Second string
        match &blocks[2].block_type {
            BlockType::String(id) => assert_eq!(
                runtime.database.strings[*id],
                "This is another line of text"
            ),
            _ => panic!("Expected String block"),
        }

        // End block
        assert!(matches!(blocks[3].block_type, BlockType::End));
    }

    #[test]
    fn can_continue_and_has_ended() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        // Should not be able to continue if the runtime is not running.
        assert_eq!(runtime.can_continue(), false);

        runtime.run();

        // Should be able to continue if the runtime is running and has not reached
        // the end of the script.
        assert_eq!(runtime.can_continue(), true);

        runtime.step(); // step to first string
        assert_eq!(runtime.can_continue(), true);

        runtime.step(); // step to second string
        assert_eq!(runtime.can_continue(), true);

        runtime.step(); // step to end

        // Should not be able to continue if the runtime is running and has reached
        // the end of the script.
        assert_eq!(runtime.has_ended(), true);
        assert_eq!(runtime.can_continue(), false);
    }

    #[test]
    fn stop_finishes_running() {
        let database = cuentitos_common::Database::default();
        let mut runtime = Runtime::new(database.clone());

        assert_eq!(runtime.running(), false);

        runtime.run();

        assert_eq!(runtime.running(), true);

        runtime.stop();

        assert_eq!(runtime.running(), false);
    }

    #[test]
    fn test_nested_block_traversal() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000009-nested-strings-with-siblings.md"),
            "nested-strings.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        // Verify Start block
        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::Start)
        ));

        // Step through each block and verify the order
        let expected_strings = vec![
            "This is the parent string",
            "This is the first child",
            "This is the first grandchild",
            "This is the second grandchild",
            "This is the second child",
            "This is the third grandchild",
            "This is the fourth grandchild",
            "This is the third child",
        ];

        for expected in expected_strings {
            runtime.step();
            if let Some(block) = runtime.current_block() {
                match block.block_type {
                    BlockType::String(id) => assert_eq!(runtime.database.strings[id], expected),
                    _ => panic!("Expected String block"),
                }
            } else {
                panic!("Expected block to be present");
            }
        }

        // Final step should reach End block
        runtime.step();
        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::End)
        ));
    }

    #[test]
    fn test_path_tracking() {
        let script = "Parent\n  Child1\n  Child2\n    Grandchild\n  Child3";
        let database = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        // Start block
        assert_eq!(runtime.current_path, vec![0]);

        // Step to Parent
        runtime.step();
        assert_eq!(runtime.current_path, vec![0, 1]);

        // Step to Child1
        runtime.step();
        assert_eq!(runtime.current_path, vec![0, 1, 2]);

        // Step to Child2
        runtime.step();
        assert_eq!(runtime.current_path, vec![0, 1, 2, 3]);

        // Step to Grandchild
        runtime.step();
        assert_eq!(runtime.current_path, vec![0, 1, 2, 3, 4]);

        // Step to Child3
        runtime.step();
        assert_eq!(runtime.current_path, vec![0, 1, 2, 3, 4, 5]);

        // Step to End
        runtime.step();
        assert_eq!(runtime.current_path, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_block_levels() {
        let script = "Parent\n  Child1\n  Child2\n    Grandchild\n  Child3";
        let database = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        // Start block (level 0)
        assert_eq!(runtime.current_block().unwrap().level, 0);

        // Parent (level 0)
        runtime.step();
        assert_eq!(runtime.current_block().unwrap().level, 0);

        // Child1 (level 1)
        runtime.step();
        assert_eq!(runtime.current_block().unwrap().level, 1);

        // Child2 (level 1)
        runtime.step();
        assert_eq!(runtime.current_block().unwrap().level, 1);

        // Grandchild (level 2)
        runtime.step();
        assert_eq!(runtime.current_block().unwrap().level, 2);

        // Child3 (level 1)
        runtime.step();
        assert_eq!(runtime.current_block().unwrap().level, 1);

        // End block (level 0)
        runtime.step();
        assert_eq!(runtime.current_block().unwrap().level, 0);
    }

    #[test]
    fn test_empty_database() {
        let database = Database::new();
        let mut runtime = Runtime::new(database);
        runtime.run();

        assert!(runtime.current_path.is_empty());
        assert_eq!(runtime.current_block(), None);
        assert!(!runtime.can_continue());
    }

    #[test]
    fn test_skip_nested_blocks() {
        let script = "Parent\n  Child1\n  Child2\n    Grandchild\n  Child3";
        let database = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        runtime.skip();

        // Verify we're at the End block
        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::End)
        ));

        // Verify current_blocks shows all blocks that were skipped
        let blocks = runtime.current_blocks();

        // Check levels and parent-child relationships
        assert_eq!(blocks[1].level, 0); // Parent
        assert_eq!(blocks[2].level, 1); // Child1
        assert_eq!(blocks[3].level, 1); // Child2
        assert_eq!(blocks[4].level, 2); // Grandchild
        assert_eq!(blocks[5].level, 1); // Child3
        assert_eq!(blocks[6].level, 0); // End

        // Verify parent-child relationships
        assert_eq!(blocks[2].parent_id, Some(1)); // Child1 -> Parent
        assert_eq!(blocks[3].parent_id, Some(1)); // Child2 -> Parent
        assert_eq!(blocks[4].parent_id, Some(3)); // Grandchild -> Child2
        assert_eq!(blocks[5].parent_id, Some(1)); // Child3 -> Parent
    }

    #[test]
    fn test_find_next_block_edge_cases() {
        // Test with a single block
        let mut database = Database::new();
        let start_block = Block::new(BlockType::Start, None, 0);
        database.add_block(start_block);

        let mut runtime = Runtime::new(database);
        runtime.run();

        // Should return None since there are no more blocks
        assert_eq!(runtime.find_next_block(), None);

        // Test with disconnected blocks (no parent-child relationships)
        let mut database = Database::new();
        database.add_block(Block::new(BlockType::Start, None, 0));
        database.add_block(Block::new(BlockType::String(0), None, 0));
        database.add_block(Block::new(BlockType::End, None, 0));

        let mut runtime = Runtime::new(database);
        runtime.run();

        // Should still be able to move to the next block
        assert_eq!(runtime.find_next_block(), Some(1));
        runtime.step();
        assert_eq!(runtime.find_next_block(), Some(2));
    }
}
