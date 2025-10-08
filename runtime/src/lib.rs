use cuentitos_common::*;

/// Represents a call frame for <-> (call and return) commands
#[derive(Debug, Clone)]
struct CallFrame {
    return_block_id: BlockId,   // Block to return to after call completes
    called_section_id: BlockId, // The section that was called
}

/// Runtime state that can be reset
#[derive(Debug, Clone)]
struct RuntimeState {
    program_counter: usize,
    previous_program_counter: usize,
    current_path: Vec<BlockId>,
    call_stack: Vec<CallFrame>,
}

impl RuntimeState {
    fn new() -> Self {
        Self {
            program_counter: 0,
            previous_program_counter: 0,
            current_path: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    fn with_start_block() -> Self {
        let mut state = Self::new();
        state.current_path.push(0); // START block
        state
    }
}

pub struct Runtime {
    pub database: Database,
    running: bool,
    state: RuntimeState,
}

impl Runtime {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            running: false,
            state: RuntimeState::new(),
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        self.reset();
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.state = RuntimeState::new();
    }

    pub fn running(&self) -> bool {
        self.running
    }

    /// Reset runtime state to initial values
    pub fn reset(&mut self) {
        self.state = if !self.database.blocks.is_empty() {
            RuntimeState::with_start_block()
        } else {
            RuntimeState::new()
        };
    }

    /// Find a section by its path string
    pub fn find_section_by_path(&self, path: &str) -> Option<SectionId> {
        self.database
            .sections
            .iter()
            .enumerate()
            .find(|(_, section)| &self.database.strings[section.path] == path)
            .map(|(section_id, _)| section_id)
    }

    pub fn can_continue(&self) -> bool {
        self.running && !self.has_ended() && !self.database.blocks.is_empty()
    }

    pub fn has_ended(&self) -> bool {
        matches!(
            self.current_block().map(|b| b.block_type),
            Some(BlockType::End)
        )
    }

    pub fn current_blocks(&self) -> Vec<Block> {
        if !self.running || self.database.blocks.is_empty() {
            Vec::new()
        } else if self.has_ended() {
            // When we've reached the end, return only blocks in the execution path
            self.state.current_path
                .iter()
                .map(|&id| self.database.blocks[id].clone())
                .collect()
        } else {
            // Return only blocks that were visited in the execution path
            self.state.current_path
                .iter()
                .map(|&id| self.database.blocks[id].clone())
                .collect()
        }
    }

    pub fn current_block(&self) -> Option<Block> {
        if self.running() && !self.database.blocks.is_empty() {
            if self.state.program_counter >= self.database.blocks.len() {
                None
            } else {
                Some(self.database.blocks[self.state.program_counter].clone())
            }
        } else {
            None
        }
    }

    // Find the next block to visit in a depth-first traversal
    fn find_next_block(&mut self) -> Option<usize> {
        if self.state.program_counter >= self.database.blocks.len() - 1 {
            return None;
        }

        let current_block = &self.database.blocks[self.state.program_counter];

        // Handle special goto variants
        match &current_block.block_type {
            // GoToAndBack: push to call stack and jump to section
            BlockType::GoToAndBack(section_id) => {
                // Check maximum call stack depth to prevent infinite loops
                const MAX_CALL_DEPTH: usize = 200;
                if self.state.call_stack.len() >= MAX_CALL_DEPTH {
                    // Print error message and terminate
                    let section = &self.database.sections[*section_id];
                    let section_name = &self.database.strings[section.name];
                    let line = current_block.line;
                    if line > 0 {
                        eprintln!(
                            "Maximum call stack depth exceeded ({} calls) due to script:{} <-> {}",
                            MAX_CALL_DEPTH, line, section_name
                        );
                    } else {
                        eprintln!(
                            "Maximum call stack depth exceeded ({} calls) due to <-> {}",
                            MAX_CALL_DEPTH, section_name
                        );
                    }
                    // Jump to END to terminate execution
                    return Some(self.database.blocks.len() - 1);
                }

                // Compute the return point (where we'd go in normal traversal)
                let return_block_id = self.compute_natural_next_block()?;

                // Get the section's block ID
                let section = &self.database.sections[*section_id];
                let target_block_id = section.block_id;

                // Push call frame
                self.state.call_stack.push(CallFrame {
                    return_block_id,
                    called_section_id: target_block_id,
                });

                return Some(target_block_id);
            }

            // GoTo: jump to section
            BlockType::GoTo(section_id) => {
                let section = &self.database.sections[*section_id];
                return Some(section.block_id);
            }

            // GoToStart: clear call stack and jump to START
            BlockType::GoToStart => {
                self.state.call_stack.clear();
                return Some(0);
            }

            // GoToRestart: clear state (except current_path) and jump to START
            BlockType::GoToRestart => {
                self.state.call_stack.clear();
                self.state.program_counter = 0;
                self.state.previous_program_counter = 0;
                // Don't touch current_path - let step() add block 0
                return Some(0);
            }

            // GoToEnd: jump to END
            BlockType::GoToEnd => {
                return Some(self.database.blocks.len() - 1);
            }

            _ => {}
        }

        // Compute natural next block (existing traversal logic)
        let natural_next = self.compute_natural_next_block()?;

        // Check if we should return from a call
        if let Some(frame) = self.state.call_stack.last() {
            // If natural_next is outside the called section's subtree, return instead
            if self.is_outside_section(natural_next, frame.called_section_id) {
                let return_id = frame.return_block_id;
                self.state.call_stack.pop();
                return Some(return_id);
            }
        }

        Some(natural_next)
    }

    // Compute the natural next block according to depth-first traversal rules
    fn compute_natural_next_block(&self) -> Option<usize> {
        let current_block = &self.database.blocks[self.state.program_counter];

        // First, try to find a child
        if !current_block.children.is_empty() {
            return Some(current_block.children[0]);
        }

        // If no children, try to find the next sibling
        let mut current_id = self.state.program_counter;
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
        if self.state.program_counter + 1 < self.database.blocks.len() {
            Some(self.state.program_counter + 1)
        } else {
            None
        }
    }

    // Check if a block is outside a section's subtree
    // A block is outside a section if the section is NOT in the block's parent chain
    fn is_outside_section(&self, block_id: BlockId, section_id: BlockId) -> bool {
        let mut current = block_id;
        loop {
            if current == section_id {
                return false; // Inside the section
            }

            match self.database.blocks[current].parent_id {
                Some(parent_id) => current = parent_id,
                None => return true, // Reached root, we're outside
            }
        }
    }

    pub fn step(&mut self) -> bool {
        if self.can_continue() {
            if let Some(next_id) = self.find_next_block() {
                self.state.previous_program_counter = self.state.program_counter;
                self.state.program_counter = next_id;
                self.state.current_path.push(next_id);
                return true;
            }
        }
        false
    }

    pub fn skip(&mut self) -> bool {
        let initial_stack_depth = self.state.call_stack.len();
        let previous_program_counter = self.state.program_counter;

        // Keep stepping until we reach END or return from current call
        while !self.has_ended() && self.can_continue() {
            self.step();

            // If we started in a call, stop when we return from it
            if initial_stack_depth > 0 && self.state.call_stack.len() < initial_stack_depth {
                break;
            }
        }

        // Set previous_program_counter to show all blocks that were skipped
        if self.state.program_counter > previous_program_counter {
            self.state.previous_program_counter = previous_program_counter;
        }

        true
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

        assert!(!runtime.running());

        runtime.run();

        assert!(runtime.running());
    }

    #[test]
    fn get_current_block() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();

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
    fn test_skip_basic() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000003-two-lines-and-skip.md"),
            "00000000003-two-lines-and-skip.md",
        );

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        // Skip should go all the way to END
        runtime.skip();

        if let Some(block) = runtime.current_block() {
            assert!(matches!(block.block_type, BlockType::End));
        } else {
            panic!("Expected to be at END block");
        }
    }

    #[test]
    fn step_moves_to_next_line() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();

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

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        runtime.run();

        runtime.skip();

        assert!(!runtime.can_continue());

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

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        runtime.skip();

        assert!(!runtime.can_continue());

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

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        // Should not be able to continue if the runtime is not running.
        assert!(!runtime.can_continue());

        runtime.run();

        // Should be able to continue if the runtime is running and has not reached
        // the end of the script.
        assert!(runtime.can_continue());

        runtime.step(); // step to first string
        assert!(runtime.can_continue());

        runtime.step(); // step to second string
        assert!(runtime.can_continue());

        runtime.step(); // step to end

        // Should not be able to continue if the runtime is running and has reached
        // the end of the script.
        assert!(runtime.has_ended());
        assert!(!runtime.can_continue());
    }

    #[test]
    fn stop_finishes_running() {
        let database = cuentitos_common::Database::default();
        let mut runtime = Runtime::new(database.clone());

        assert!(!runtime.running());

        runtime.run();

        assert_eq!(runtime.running(), true);

        runtime.stop();

        assert!(!runtime.running());
    }

    #[test]
    fn test_nested_block_traversal() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000009-nested-strings-with-siblings.md"),
            "nested-strings.md",
        );

        let (database, _warnings) = cuentitos_parser::parse(&test_case.script).unwrap();
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
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
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
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
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
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
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

    #[test]
    fn test_jump_backward_loop() {
        // Test that jumping backward creates a proper loop
        let script =
            "# Section A\nText in A\n\n# Section B\nText in B\n\n# Section C\n-> Section A";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        // Should loop: Start -> A -> Text -> B -> Text -> C -> A -> Text -> B -> Text -> C -> A ...
        // Execute 20 steps to verify looping works
        for _i in 0..20 {
            assert!(runtime.can_continue(), "Runtime should continue looping");
            runtime.step();
        }
    }

    #[test]
    fn test_jump_to_parent_loop() {
        // Test that jumping to parent using .. creates a proper loop
        // This is test 038 from compatibility tests
        let script = "# Parent\nText in parent\n  ## Child\n  Text in child\n  -> ..";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();

        let mut runtime = Runtime::new(database);
        runtime.run();

        // Test 038 has 20 'n' commands, so we should be able to step 20 times
        for i in 0..20 {
            assert!(
                runtime.can_continue(),
                "Runtime should continue looping at step {}",
                i
            );
            let stepped = runtime.step();
            assert!(stepped, "Step should succeed at iteration {}", i);
        }

        // After 20 steps, current_path should have 21 blocks (START + 20 steps)
        assert_eq!(
            runtime.current_blocks().len(),
            21,
            "Should have 21 blocks in path after 20 steps"
        );
    }

    #[test]
    fn test_recursive_call_with_skip() {
        // Test that recursive calls hit MAX_CALL_DEPTH and end
        let script = "# Loop\nIteration text\n<-> .";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();

        let mut runtime = Runtime::new(database);
        runtime.run();

        // First step: START -> Loop
        runtime.step();

        // Skip should hit MAX_CALL_DEPTH and jump to END
        runtime.skip();

        // Should be at END after hitting the depth limit
        if let Some(block) = runtime.current_block() {
            assert!(
                matches!(block.block_type, BlockType::End),
                "Should be at END after hitting MAX_CALL_DEPTH"
            );
        }

        // Verify runtime has ended
        assert!(runtime.has_ended(), "Runtime should have ended");
    }

    #[test]
    fn test_basic_call_and_return_with_skip() {
        let script =
            "# Section A\nText in A\n<-> Section B\nText after call in A\n\n# Section B\nText in B";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();

        let mut runtime = Runtime::new(database);
        runtime.run();

        // Skip should execute everything
        runtime.skip();

        // Should have visited: START, Section A, Text in A, <->, Section B, Text in B, Text after call, END
        let blocks = runtime.current_blocks();
        println!("Blocks visited: {}", blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            println!("{}: {:?}", i, block.block_type);
        }

        // Should be at END
        assert!(runtime.has_ended());
    }

    #[test]
    fn test_basic_call_and_return() {
        let script =
            "# Section A\nText in A\n<-> Section B\nText after call in A\n\n# Section B\nText in B";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();

        let mut runtime = Runtime::new(database);
        runtime.run();

        // START
        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::Start)
        ));

        // Step to Section A
        runtime.step();
        if let Some(block) = runtime.current_block() {
            assert!(matches!(block.block_type, BlockType::Section { .. }));
        }

        // Step to "Text in A"
        runtime.step();
        if let Some(block) = runtime.current_block() {
            if let BlockType::String(id) = block.block_type {
                assert_eq!(runtime.database.strings[id], "Text in A");
            } else {
                panic!("Expected String block");
            }
        }

        // Step to <-> Section B block
        runtime.step();
        // Should now be AT the GoToSectionAndBack block
        if let Some(block) = runtime.current_block() {
            assert!(matches!(
                block.block_type,
                BlockType::GoToSectionAndBack { .. }
            ));
        }

        // Step again to jump to Section B
        runtime.step();
        // Should now be at Section B
        if let Some(block) = runtime.current_block() {
            if let BlockType::Section { display_name, .. } = &block.block_type {
                assert_eq!(display_name, "Section B");
            } else {
                panic!("Expected Section B, got {:?}", block.block_type);
            }
        } else {
            panic!("Expected block");
        }

        // Step to "Text in B"
        runtime.step();
        if let Some(block) = runtime.current_block() {
            if let BlockType::String(id) = block.block_type {
                assert_eq!(runtime.database.strings[id], "Text in B");
            } else {
                panic!("Expected String block");
            }
        }

        // Step - should return to "Text after call in A"
        runtime.step();
        if let Some(block) = runtime.current_block() {
            if let BlockType::String(id) = block.block_type {
                assert_eq!(runtime.database.strings[id], "Text after call in A");
            } else {
                panic!(
                    "Expected 'Text after call in A', got {:?}",
                    block.block_type
                );
            }
        } else {
            panic!("Expected block after return");
        }
    }
}
