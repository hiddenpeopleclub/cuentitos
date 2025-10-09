use cuentitos_common::*;

pub mod error;
pub use error::RuntimeError;

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
    waiting_for_option_selection: bool,
    current_options: Vec<BlockId>, // IDs of available option blocks
}

impl RuntimeState {
    fn new() -> Self {
        Self {
            program_counter: 0,
            previous_program_counter: 0,
            current_path: Vec::new(),
            call_stack: Vec::new(),
            waiting_for_option_selection: false,
            current_options: Vec::new(),
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

    /// Find a section by its path string, resolving relative paths from current context
    pub fn find_section_by_path(&self, path: &str) -> Result<ResolvedPath, RuntimeError> {
        // Find the containing section based on current program counter
        let containing_section = self.find_containing_section(self.state.program_counter);

        // Use PathResolver to resolve the path
        let resolver = PathResolver::new(&self.database, &self.database.section_registry);

        resolver
            .resolve_path(path, containing_section)
            .map_err(RuntimeError::from)
    }

    /// Find the nearest ancestor section that contains the given block
    fn find_containing_section(&self, block_id: BlockId) -> Option<BlockId> {
        let mut current_id = block_id;

        // Walk up parents until we find a Section block
        while let Some(parent_id) = self
            .database
            .blocks
            .get(current_id)
            .and_then(|b| b.parent_id)
        {
            if matches!(
                self.database.blocks[parent_id].block_type,
                BlockType::Section(_)
            ) {
                return Some(parent_id);
            }
            current_id = parent_id;
        }

        None
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
            self.state
                .current_path
                .iter()
                .map(|&id| self.database.blocks[id].clone())
                .collect()
        } else {
            // Return only blocks that were visited in the execution path
            self.state
                .current_path
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

    /// Returns true if the runtime is waiting for user to select an option
    pub fn is_waiting_for_option(&self) -> bool {
        self.state.waiting_for_option_selection
    }

    /// Returns the current available options as (option_number, string_id) pairs
    /// Option numbers start at 1
    pub fn get_current_options(&self) -> Vec<(usize, StringId)> {
        self.state
            .current_options
            .iter()
            .enumerate()
            .filter_map(|(i, &block_id)| {
                if let BlockType::Option(string_id) = self.database.blocks[block_id].block_type {
                    Some((i + 1, string_id)) // Numbers start at 1
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the current path of executed blocks
    pub fn current_path(&self) -> &[BlockId] {
        &self.state.current_path
    }

    /// Select an option by its number (1-based)
    /// Returns Ok(()) if successful, Err(message) if invalid choice
    pub fn select_option(&mut self, choice: usize) -> Result<(), String> {
        if !self.state.waiting_for_option_selection {
            return Err("Not waiting for option selection".to_string());
        }

        if choice == 0 || choice > self.state.current_options.len() {
            return Err(format!("Invalid option: {}", choice));
        }

        // Get the selected option block ID (choice is 1-based, vec is 0-based)
        let selected_option_id = self.state.current_options[choice - 1];

        // Add selected option to execution path
        self.state.current_path.push(selected_option_id);

        // Move program counter to the selected option
        self.state.program_counter = selected_option_id;

        // Clear option selection state
        self.state.waiting_for_option_selection = false;
        self.state.current_options.clear();

        Ok(())
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
        // Special handling: if we're inside an option, skip all sibling options when exiting
        let mut current_id = self.state.program_counter;
        while let Some(parent_id) = self.database.blocks[current_id].parent_id {
            let parent = &self.database.blocks[parent_id];
            let current_index = parent
                .children
                .iter()
                .position(|&id| id == current_id)
                .unwrap();

            // Look for next sibling
            for sibling_index in (current_index + 1)..parent.children.len() {
                let sibling_id = parent.children[sibling_index];
                let sibling_is_option = matches!(
                    self.database.blocks[sibling_id].block_type,
                    BlockType::Option(_)
                );

                // Check if we're inside an option's subtree (recalculate for current_id)
                let inside_option = self.is_inside_option_subtree(current_id);

                // If we're exiting an option and this sibling is an option, skip it
                if inside_option && sibling_is_option {
                    continue;
                }

                // Found a valid next sibling
                return Some(sibling_id);
            }

            // No valid sibling found, move up to parent
            current_id = parent_id;
        }

        // Fallback: if we've exhausted the tree, try sequential next block
        // But skip over option blocks AND their subtrees if we're coming from inside an option
        let inside_option = self.is_inside_option_subtree(self.state.program_counter);
        let mut next_id = self.state.program_counter + 1;
        while next_id < self.database.blocks.len() {
            // If we're inside an option and the next block is an option or inside an option, skip it
            if inside_option && self.is_inside_option_subtree(next_id) {
                next_id += 1;
                continue;
            }
            return Some(next_id);
        }
        None
    }

    // Check if we're currently inside an option's subtree (including the option itself)
    fn is_inside_option_subtree(&self, block_id: BlockId) -> bool {
        let mut current_id = block_id;
        loop {
            if matches!(
                self.database.blocks[current_id].block_type,
                BlockType::Option(_)
            ) {
                return true;
            }
            match self.database.blocks[current_id].parent_id {
                Some(parent_id) => current_id = parent_id,
                None => return false,
            }
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
                // Check if the next block is an option
                if matches!(
                    self.database.blocks[next_id].block_type,
                    BlockType::Option(_)
                ) {
                    // Collect all option siblings
                    self.collect_options_at(next_id);
                    return false; // Stop stepping, wait for user choice
                }

                self.state.previous_program_counter = self.state.program_counter;
                self.state.program_counter = next_id;
                self.state.current_path.push(next_id);
                return true;
            }
        }
        false
    }

    /// Collect all option siblings starting from the first option
    fn collect_options_at(&mut self, first_option_id: BlockId) {
        self.state.current_options.clear();
        self.state.waiting_for_option_selection = true;

        // Get parent to find all option siblings
        if let Some(parent_id) = self.database.blocks[first_option_id].parent_id {
            let parent = &self.database.blocks[parent_id];

            // Find all consecutive option children
            for &child_id in &parent.children {
                if matches!(
                    self.database.blocks[child_id].block_type,
                    BlockType::Option(_)
                ) {
                    self.state.current_options.push(child_id);
                } else if !self.state.current_options.is_empty() {
                    // Stop when we hit a non-option after options have started
                    break;
                }
            }
        }
    }

    pub fn skip(&mut self) -> bool {
        let initial_stack_depth = self.state.call_stack.len();
        let previous_program_counter = self.state.program_counter;

        // Keep stepping until we reach END, return from current call, or hit options
        while !self.has_ended() && self.can_continue() && !self.state.waiting_for_option_selection {
            self.step();

            // If we started in a call, stop when we return from it
            if initial_stack_depth > 0 && self.state.call_stack.len() < initial_stack_depth {
                break;
            }

            // Stop if we're now waiting for option selection
            if self.state.waiting_for_option_selection {
                break;
            }
        }

        // Set previous_program_counter to show all blocks that were skipped
        if self.state.program_counter > previous_program_counter {
            self.state.previous_program_counter = previous_program_counter;
        }

        true
    }

    /// Jump to a section (permanent jump, does not return)
    pub fn goto_section(&mut self, section_id: SectionId) -> Result<(), RuntimeError> {
        if !self.running {
            return Err(RuntimeError::NotRunning);
        }

        let section = &self.database.sections[section_id];
        let target_block_id = section.block_id;

        // Set program counter to the section block
        self.state.program_counter = target_block_id;
        // Add to current path
        self.state.current_path.push(target_block_id);

        Ok(())
    }

    /// Jump to a section and return (call and return)
    pub fn goto_and_back_section(&mut self, section_id: SectionId) -> Result<(), RuntimeError> {
        if !self.running {
            return Err(RuntimeError::NotRunning);
        }

        // Check maximum call stack depth
        const MAX_CALL_DEPTH: usize = 200;
        if self.state.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(RuntimeError::InvalidPath {
                message: format!(
                    "Maximum call stack depth exceeded ({} calls)",
                    MAX_CALL_DEPTH
                ),
            });
        }

        let section = &self.database.sections[section_id];
        let target_block_id = section.block_id;

        // Compute where we would return to (natural next block)
        let return_block_id =
            self.compute_natural_next_block()
                .ok_or_else(|| RuntimeError::InvalidPath {
                    message: "Cannot compute return point for call".to_string(),
                })?;

        // Push call frame
        self.state.call_stack.push(CallFrame {
            return_block_id,
            called_section_id: target_block_id,
        });

        // Set program counter to the section block
        self.state.program_counter = target_block_id;
        // Add to current path
        self.state.current_path.push(target_block_id);

        Ok(())
    }

    /// Jump to START (clears call stack, restarts from beginning)
    pub fn goto_start(&mut self) -> Result<(), RuntimeError> {
        if !self.running {
            return Err(RuntimeError::NotRunning);
        }

        // Clear call stack
        self.state.call_stack.clear();

        // Jump to block 0 (START)
        self.state.program_counter = 0;
        self.state.current_path.push(0);

        Ok(())
    }

    /// Jump to RESTART (resets state and restarts from beginning)
    pub fn goto_restart(&mut self) -> Result<(), RuntimeError> {
        if !self.running {
            return Err(RuntimeError::NotRunning);
        }

        // Clear call stack
        self.state.call_stack.clear();

        // Reset counters
        self.state.program_counter = 0;
        self.state.previous_program_counter = 0;

        // Jump to block 0 (START)
        self.state.current_path.push(0);

        Ok(())
    }

    /// Jump to END
    pub fn goto_end(&mut self) -> Result<(), RuntimeError> {
        if !self.running {
            return Err(RuntimeError::NotRunning);
        }

        // Jump to last block (END)
        let end_block_id = self.database.blocks.len() - 1;
        self.state.program_counter = end_block_id;
        self.state.current_path.push(end_block_id);

        Ok(())
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

        assert_eq!(runtime.state.program_counter, 2);

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
        assert_eq!(runtime.state.current_path, vec![0]);

        // Step to Parent
        runtime.step();
        assert_eq!(runtime.state.current_path, vec![0, 1]);

        // Step to Child1
        runtime.step();
        assert_eq!(runtime.state.current_path, vec![0, 1, 2]);

        // Step to Child2
        runtime.step();
        assert_eq!(runtime.state.current_path, vec![0, 1, 2, 3]);

        // Step to Grandchild
        runtime.step();
        assert_eq!(runtime.state.current_path, vec![0, 1, 2, 3, 4]);

        // Step to Child3
        runtime.step();
        assert_eq!(runtime.state.current_path, vec![0, 1, 2, 3, 4, 5]);

        // Step to End
        runtime.step();
        assert_eq!(runtime.state.current_path, vec![0, 1, 2, 3, 4, 5, 6]);
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

        assert!(runtime.state.current_path.is_empty());
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
    fn test_cli_goto_and_back() {
        // Test CLI goto_and_back_section method
        let script = "# Section A\nText in A\nMore text in A\n\n# Section B\nText in B";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();

        let mut runtime = Runtime::new(database);
        runtime.run();

        // Step to Section A
        runtime.step();
        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::Section(_))
        ));

        // Step to "Text in A"
        runtime.step();

        // Now call Section B using CLI command
        let resolved = runtime.find_section_by_path("Section B").unwrap();
        if let ResolvedPath::Section(section_id) = resolved {
            runtime.goto_and_back_section(section_id).unwrap();
        } else {
            panic!("Expected Section");
        }

        // Should now be at Section B
        assert!(matches!(
            runtime.current_block().map(|b| b.block_type),
            Some(BlockType::Section(_))
        ));

        // Step to "Text in B"
        runtime.step();

        // Verify we're at "Text in B"
        if let Some(block) = runtime.current_block() {
            if let BlockType::String(id) = block.block_type {
                assert_eq!(runtime.database.strings[id], "Text in B");
            } else {
                panic!("Expected String block");
            }
        }

        // Step again - should return to "More text in A"
        runtime.step();
        if let Some(block) = runtime.current_block() {
            if let BlockType::String(id) = block.block_type {
                assert_eq!(runtime.database.strings[id], "More text in A");
            } else {
                panic!("Expected 'More text in A', got {:?}", block.block_type);
            }
        }
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
        // Should now be AT the GoToAndBack block
        if let Some(block) = runtime.current_block() {
            assert!(matches!(block.block_type, BlockType::GoToAndBack(_)));
        }

        // Step again to jump to Section B
        runtime.step();
        // Should now be at Section B
        if let Some(block) = runtime.current_block() {
            if let BlockType::Section(section_id) = &block.block_type {
                let section_name =
                    &runtime.database.strings[runtime.database.sections[*section_id].name];
                assert_eq!(section_name, "Section B");
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

    #[test]
    fn test_option_skip_doesnt_reencounter_options() {
        // Test that after selecting an option and skipping, we don't encounter sibling options
        let script = "Question\n  * Option 1\n    Content 1\n  * Option 2\n    Content 2";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        // Step to "Question"
        runtime.step();

        // Step should hit options
        runtime.step();
        assert!(runtime.is_waiting_for_option());

        // Select first option
        runtime.select_option(1).unwrap();
        assert!(!runtime.is_waiting_for_option());

        // Skip should go to END without re-encountering options
        runtime.skip();
        assert!(
            !runtime.is_waiting_for_option(),
            "Should not be waiting for options after skip"
        );
        assert!(runtime.has_ended(), "Should have reached END");
    }
}
