use cuentitos_common::*;
use std::path::PathBuf;

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
    last_error: Option<RuntimeError>,
    /// Current variable values, aligned index-for-index with
    /// `Database.variables`. `variable_values[i]` is the current value of the
    /// variable declared at position `i`. Always reinitialized from declared
    /// defaults on every `reset()` (and therefore on every `run()`).
    ///
    /// Typed so that adding new `Value` variants (bool, float, string)
    /// is strictly additive — no storage migration needed.
    variable_values: Vec<Value>,
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
            last_error: None,
            variable_values: Vec::new(),
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
    /// Optional source path used to format runtime errors as
    /// `<file>:<line>: RUNTIME ERROR: ...`. None when the database came from
    /// an in-memory script (e.g. unit tests).
    file_path: Option<PathBuf>,
}

impl Runtime {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            running: false,
            state: RuntimeState::new(),
            file_path: None,
        }
    }

    /// Construct a runtime that knows the source script path. Required for
    /// runtime arithmetic errors to include `<file>:<line>:` prefixes that
    /// match the parse-time error format.
    pub fn with_file(database: Database, file_path: PathBuf) -> Self {
        Self {
            database,
            running: false,
            state: RuntimeState::new(),
            file_path: Some(file_path),
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

    /// Reset runtime state to initial values.
    ///
    /// Unconditionally reinitializes `variable_values` from each variable's
    /// declared default — any runtime mutations made via
    /// [`Runtime::set_variable_value`] since the last reset are discarded.
    pub fn reset(&mut self) {
        self.state = if !self.database.blocks.is_empty() {
            RuntimeState::with_start_block()
        } else {
            RuntimeState::new()
        };
        self.state.variable_values = self
            .database
            .variables
            .iter()
            .map(|v| v.initial_value())
            .collect();
    }

    /// Returns the current value of every declared variable, in declaration order.
    pub fn variable_values(&self) -> &[Value] {
        &self.state.variable_values
    }

    /// Returns the current value of the variable with the given name.
    pub fn variable_value(&self, name: &str) -> Option<&Value> {
        self.database
            .variable_id(name)
            .and_then(|id| self.state.variable_values.get(id))
    }

    /// Sets a variable by name; returns
    /// [`RuntimeError::UndefinedVariable`] if no variable with that name has
    /// been declared, or [`RuntimeError::VariableTypeMismatch`] if `value`'s
    /// kind differs from the variable's declared kind.
    ///
    /// This is introduced now so `set` execution and external mutations share
    /// a single stable mutation point.
    pub fn set_variable_value(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        let id =
            self.database
                .variable_id(name)
                .ok_or_else(|| RuntimeError::UndefinedVariable {
                    name: name.to_string(),
                })?;
        // Reject writes that don't match the declared kind. Today there's
        // only one `Value` variant, so this is unreachable — it exists as a
        // guard for future kinds (bool/float/string) so the setter can't
        // silently clobber the declared kind.
        if self.state.variable_values[id].kind() != value.kind() {
            return Err(RuntimeError::VariableTypeMismatch {
                name: name.to_string(),
            });
        }
        self.state.variable_values[id] = value;
        Ok(())
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

        if matches!(
            self.database.blocks[current_id].block_type,
            BlockType::Section(_)
        ) {
            return Some(current_id);
        }

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
        } else {
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

    /// Returns and clears the last runtime error, if any
    pub fn take_last_error(&mut self) -> Option<RuntimeError> {
        self.state.last_error.take()
    }

    /// Non-consuming peek at the last runtime error.
    pub fn has_error(&self) -> bool {
        self.state.last_error.is_some()
    }

    /// Returns the block IDs of the current available options
    pub fn get_current_option_block_ids(&self) -> &[BlockId] {
        &self.state.current_options
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
                    let section = &self.database.sections[*section_id];
                    let section_name = &self.database.strings[section.name];
                    let line = current_block.line;
                    let message = if line > 0 {
                        format!(
                            "Maximum call stack depth exceeded ({} calls) due to script:{} <-> {}",
                            MAX_CALL_DEPTH, line, section_name
                        )
                    } else {
                        format!(
                            "Maximum call stack depth exceeded ({} calls) due to <-> {}",
                            MAX_CALL_DEPTH, section_name
                        )
                    };
                    self.state.last_error = Some(RuntimeError::InvalidPath { message });
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
        if !self.can_continue() {
            return false;
        }
        // Advance one user-visible block. Silent side-effect blocks
        // (`Set`, `Req`) are transparently traversed so a single `n` in
        // the CLI lands on the next narratively-visible block.
        let mut advanced = false;
        loop {
            let Some(next_id) = self.find_next_block() else {
                return advanced;
            };

            // Before entering `next_id`, evaluate any `req` children that
            // gate it. A failing `req` skips `next_id` and its entire
            // subtree without rendering anything; an evaluation error
            // (overflow, div-by-zero) propagates as a runtime error.
            match self.evaluate_requirement_gating(next_id) {
                Ok(true) => {}
                Ok(false) => {
                    let skip_to = self.last_descendant(next_id);
                    self.state.previous_program_counter = self.state.program_counter;
                    self.state.program_counter = skip_to;
                    if !self.can_continue() {
                        return advanced;
                    }
                    continue;
                }
                Err(err) => {
                    self.state.last_error = Some(err);
                    let end_id = self.database.blocks.len() - 1;
                    self.state.previous_program_counter = self.state.program_counter;
                    self.state.program_counter = end_id;
                    return advanced;
                }
            }

            // Stop on options — the CLI must prompt the user.
            if matches!(
                self.database.blocks[next_id].block_type,
                BlockType::Option(_)
            ) {
                self.collect_options_at(next_id);
                return advanced;
            }

            // Apply `set` side effects when stepping *onto* the block.
            // Arithmetic errors halt execution: jump PC to END (to
            // terminate the outer loop) but do NOT push END onto
            // current_path (so render_path_from won't print END).
            if let BlockType::Set(set_id) = self.database.blocks[next_id].block_type {
                let line = self.database.blocks[next_id].line;
                if let Err(err) = self.apply_set(set_id, line) {
                    self.state.last_error = Some(err);
                    let end_id = self.database.blocks.len() - 1;
                    self.state.previous_program_counter = self.state.program_counter;
                    self.state.program_counter = end_id;
                    return advanced;
                }
            }

            self.state.previous_program_counter = self.state.program_counter;
            self.state.program_counter = next_id;
            self.state.current_path.push(next_id);
            advanced = true;

            // Continue past silent blocks so a single `step()` lands on the
            // next visible block. Any non-silent block ends the step.
            if !Self::is_silent_block(&self.database.blocks[next_id].block_type) {
                return true;
            }
            if !self.can_continue() {
                return advanced;
            }
        }
    }

    /// Blocks that produce no narrative output and should be traversed
    /// transparently by a single `step()`. `Set` mutates a variable;
    /// `Requirement` gates its parent and is itself never rendered.
    fn is_silent_block(block_type: &BlockType) -> bool {
        matches!(block_type, BlockType::Set(_) | BlockType::Requirement(_))
    }

    /// Walk to the rightmost descendant of `block_id` so we can land
    /// `program_counter` past a gated subtree. The next call to
    /// `find_next_block` then resumes traversal at the gated block's
    /// next sibling.
    fn last_descendant(&self, block_id: BlockId) -> BlockId {
        let mut current = block_id;
        loop {
            let block = &self.database.blocks[current];
            match block.children.last() {
                Some(&last_child) => current = last_child,
                None => return current,
            }
        }
    }

    /// Evaluate every `req` child of `block_id` against the current
    /// variable values. Multiple `req` siblings act as implicit AND;
    /// short-circuits on the first failure so trailing `req`s with
    /// runtime errors don't fire when an earlier sibling already
    /// disqualified the parent.
    ///
    /// Note: gating runs before the `Option` stop in `step()`, so failing-req
    /// options are silently filtered out of the choice list — the runtime
    /// walks past them and either presents the next passing option or, if
    /// none remain, lands on the post-options content.
    fn evaluate_requirement_gating(&self, block_id: BlockId) -> Result<bool, RuntimeError> {
        // The lookup closure captures `&self.state.variable_values` and
        // is identical for every sibling `req`. Build it once outside
        // the loop — same shape `apply_set` uses for its one-shot eval.
        let lookup = cuentitos_common::variable_lookup(&self.state.variable_values);
        for &child_id in &self.database.blocks[block_id].children {
            let BlockType::Requirement(requirement_id) = self.database.blocks[child_id].block_type
            else {
                continue;
            };
            let expression = &self.database.requirements[requirement_id];
            let line = self.database.blocks[child_id].line;
            // `BooleanExpression::evaluate` short-circuits internally for
            // `and`/`or`/`not`. Sibling `req`s remain implicitly ANDed by
            // the loop here — failing one short-circuits the whole gate.
            let outcome = match expression.evaluate(&lookup) {
                Ok(value) => value,
                Err(err) => return Err(self.evaluation_error_to_runtime(err, line)),
            };
            if !outcome {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Translate an [`EvaluationError`] into a [`RuntimeError`] using the
    /// file/line context for this runtime. Today only `DivisionByZero` and
    /// `Overflow` are reachable through normal scripts; `TypeMismatch`
    /// is unreachable while `Value` has the single `Integer` variant
    /// because parser-time inference rejects mixed-kind operands, but
    /// it is plumbed through to a typed [`RuntimeError::EvaluationTypeMismatch`]
    /// so the runtime path doesn't `panic!` once a second `Value`
    /// variant lands. TODO: covered once Float/String land in `Value`.
    fn evaluation_error_to_runtime(&self, err: EvaluationError, line: usize) -> RuntimeError {
        match err {
            EvaluationError::DivisionByZero => RuntimeError::DivisionByZero {
                file: self.file_path.clone(),
                line,
            },
            EvaluationError::Overflow => RuntimeError::IntegerOverflow {
                file: self.file_path.clone(),
                line,
            },
            EvaluationError::FloatOverflow => RuntimeError::FloatOverflow {
                file: self.file_path.clone(),
                line,
            },
            EvaluationError::TypeMismatch { expected, found } => {
                RuntimeError::EvaluationTypeMismatch {
                    expected,
                    found,
                    file: self.file_path.clone(),
                    line,
                }
            }
            EvaluationError::UnsetEnum { variable } => RuntimeError::UnsetEnumRead {
                name: self.database.variables[variable].name.clone(),
                file: self.file_path.clone(),
                line,
            },
        }
    }

    /// Evaluate the RHS expression of a `set` against current variable values
    /// and apply the assignment operator to the target variable.
    fn apply_set(&mut self, set_id: SetId, line: usize) -> Result<(), RuntimeError> {
        // Read inputs through immutable borrows so we don't clone the
        // expression AST or the variable-values vector. The borrows end
        // before the final write to `self.state.variable_values`.
        let (operator, variable_id, rhs_value) = {
            let statement = &self.database.sets[set_id];
            let lookup = cuentitos_common::variable_lookup(&self.state.variable_values);
            let rhs = match cuentitos_common::evaluate(&statement.expression, &lookup) {
                Ok(value) => value.into_owned(),
                Err(err) => return Err(self.evaluation_error_to_runtime(err, line)),
            };
            (statement.operator, statement.variable_id, rhs)
        };

        // Plain `Assign` overwrites the LHS unconditionally and never reads
        // its prior value. Compound operators read the LHS, then reduce the
        // pair via `BinaryOperator::apply` so checked arithmetic is shared
        // with `Expression::Binary`.
        if !operator.is_compound() {
            self.state.variable_values[variable_id] = rhs_value;
            return Ok(());
        }
        let binary_operator = match operator {
            AssignmentOperator::Assign => unreachable!("handled by the early return above"),
            AssignmentOperator::AddAssign => BinaryOperator::Add,
            AssignmentOperator::SubtractAssign => BinaryOperator::Subtract,
            AssignmentOperator::MultiplyAssign => BinaryOperator::Multiply,
            AssignmentOperator::DivideAssign => BinaryOperator::Divide,
        };
        let new_value = binary_operator
            .apply(&self.state.variable_values[variable_id], &rhs_value)
            .map_err(|err| self.evaluation_error_to_runtime(err, line))?;
        self.state.variable_values[variable_id] = new_value;
        Ok(())
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

            // Stop on a runtime error so the caller can surface it before
            // any further blocks render.
            if self.state.last_error.is_some() {
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
    fn variable_values_initialize_from_defaults_on_run() {
        let script = "--- variables\nint a = 5\nint b\nint c = a + 2\n---\n\nStory.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        assert_eq!(database.variables.len(), 3);

        let mut runtime = Runtime::new(database);
        runtime.run();

        assert_eq!(
            runtime.variable_values(),
            &[Value::Integer(5), Value::Integer(0), Value::Integer(7),]
        );
        assert_eq!(runtime.variable_value("a"), Some(&Value::Integer(5)));
        assert_eq!(runtime.variable_value("b"), Some(&Value::Integer(0)));
        assert_eq!(runtime.variable_value("c"), Some(&Value::Integer(7)));
        assert_eq!(runtime.variable_value("missing"), None);
    }

    #[test]
    fn set_variable_value_updates_state() {
        let script = "--- variables\nint a = 5\n---\n\nStory.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        runtime.set_variable_value("a", Value::Integer(42)).unwrap();
        assert_eq!(runtime.variable_value("a"), Some(&Value::Integer(42)));

        assert!(matches!(
            runtime.set_variable_value("missing", Value::Integer(1)),
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }

    #[test]
    fn reset_restores_declared_defaults_after_mutation() {
        // `Runtime::reset` (and therefore `run`) must reinitialize variable
        // values from declared defaults, discarding any prior mutations
        // performed via `set_variable_value`.
        let script = "--- variables\nint a = 5\nint b = 10\n---\n\nStory.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        runtime.set_variable_value("a", Value::Integer(99)).unwrap();
        runtime.set_variable_value("b", Value::Integer(-1)).unwrap();
        assert_eq!(runtime.variable_value("a"), Some(&Value::Integer(99)));
        assert_eq!(runtime.variable_value("b"), Some(&Value::Integer(-1)));

        runtime.reset();

        assert_eq!(runtime.variable_value("a"), Some(&Value::Integer(5)));
        assert_eq!(runtime.variable_value("b"), Some(&Value::Integer(10)));

        runtime
            .set_variable_value("a", Value::Integer(123))
            .unwrap();
        runtime.run();
        assert_eq!(runtime.variable_value("a"), Some(&Value::Integer(5)));
    }

    /// Helper: parse `script`, run to END, return the final value of `name`.
    fn run_and_read(script: &str, name: &str) -> Option<Value> {
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        runtime.variable_value(name).cloned()
    }

    #[test]
    fn apply_set_plain_assign() {
        let script = "--- variables\nint x = 0\n---\nset x = 7\nDone.";
        assert_eq!(run_and_read(script, "x"), Some(Value::Integer(7)));
    }

    #[test]
    fn apply_set_add_assign() {
        let script = "--- variables\nint x = 5\n---\nset x += 3\nDone.";
        assert_eq!(run_and_read(script, "x"), Some(Value::Integer(8)));
    }

    #[test]
    fn apply_set_sub_assign() {
        let script = "--- variables\nint x = 10\n---\nset x -= 4\nDone.";
        assert_eq!(run_and_read(script, "x"), Some(Value::Integer(6)));
    }

    #[test]
    fn apply_set_mul_assign() {
        let script = "--- variables\nint x = 3\n---\nset x *= 4\nDone.";
        assert_eq!(run_and_read(script, "x"), Some(Value::Integer(12)));
    }

    #[test]
    fn apply_set_div_assign() {
        let script = "--- variables\nint x = 20\n---\nset x /= 4\nDone.";
        assert_eq!(run_and_read(script, "x"), Some(Value::Integer(5)));
    }

    #[test]
    fn apply_set_div_by_zero_literal() {
        let script = "--- variables\nint x = 10\n---\nset x /= 0\nDone.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(matches!(
            runtime.take_last_error(),
            Some(RuntimeError::DivisionByZero { .. })
        ));
    }

    #[test]
    fn apply_set_div_by_zero_through_expression() {
        let script = "--- variables\nint x = 10\nint y = 0\n---\nset x = x / y\nDone.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(matches!(
            runtime.take_last_error(),
            Some(RuntimeError::DivisionByZero { .. })
        ));
    }

    #[test]
    fn req_reading_unset_enum_is_a_runtime_error() {
        // `mood` is never assigned, so the gating `req` reads an unset enum —
        // a runtime error rather than a silent non-match.
        let script = "--- variables\nenum mood = happy, sad\n---\nFirst.\n  req mood = happy\n";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert_eq!(
            runtime.take_last_error(),
            Some(RuntimeError::UnsetEnumRead {
                name: "mood".to_string(),
                file: None,
                line: 5,
            })
        );
    }

    #[test]
    fn req_assigned_enum_does_not_error() {
        // Once `mood` is set, the same gating `req` evaluates cleanly.
        let script =
            "--- variables\nenum mood = happy, sad\n---\nset mood = happy\nFirst.\n  req mood = happy\n";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert_eq!(runtime.take_last_error(), None);
    }

    #[test]
    fn unset_enum_read_error_displays_variable_name() {
        let error = RuntimeError::UnsetEnumRead {
            name: "mood".to_string(),
            file: Some(std::path::PathBuf::from("foo.cuentitos")),
            line: 7,
        };
        assert_eq!(
            error.to_string(),
            "foo.cuentitos:7: RUNTIME ERROR: Cannot read unset enum variable 'mood'."
        );
    }

    #[test]
    fn apply_set_add_assign_overflow_at_i64_max() {
        // x = i64::MAX, then x += 1 → overflow.
        let script = "--- variables\nint x = 9223372036854775807\n---\nset x += 1\nDone.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(matches!(
            runtime.take_last_error(),
            Some(RuntimeError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn apply_set_mul_assign_overflow_at_i64_max() {
        let script = "--- variables\nint x = 9223372036854775807\n---\nset x *= 2\nDone.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(matches!(
            runtime.take_last_error(),
            Some(RuntimeError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn apply_set_multi_read_uses_pre_assignment_value() {
        // For `set x = x*2 + x` with x=3, every read of `x` on the RHS sees
        // the pre-assignment value (3), so the result is 3*2 + 3 = 9 — not
        // 6 + 6 = 12, which would imply the first read writes back before
        // the second.
        let script = "--- variables\nint x = 3\n---\nset x = x * 2 + x\nDone.";
        assert_eq!(run_and_read(script, "x"), Some(Value::Integer(9)));
    }

    #[test]
    fn apply_set_plain_assign_skips_lhs_read() {
        // Plain `Assign` overwrites the LHS unconditionally and must not
        // read its prior value. Today every `Value` is `Integer(_)` so the
        // distinction is invisible — calling `apply_set` directly pins the
        // unit-level contract so the skip survives future refactors and
        // gates the kind-mismatch check once a second `Value` variant lands.
        let script = "--- variables\nint x = 5\n---\nset x = 7\nDone.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();

        assert_eq!(runtime.database.sets.len(), 1);
        runtime
            .apply_set(0, 0)
            .expect("plain assign should not error");
        assert_eq!(runtime.variable_value("x"), Some(&Value::Integer(7)));
    }

    #[test]
    fn apply_set_negation_of_i64_min_overflows_at_runtime() {
        // `-9223372036854775808` is folded into `Lit(i64::MIN)` at parse
        // time, but `-x` for `x = i64::MIN` is lowered to `0 - x` and must
        // overflow at runtime. Pinning the parse/runtime split here so a
        // future change to the lowering doesn't silently accept it.
        let script =
            "--- variables\nint x = 0\n---\nset x = -9223372036854775808\nset x = -x\nDone.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(matches!(
            runtime.take_last_error(),
            Some(RuntimeError::IntegerOverflow { .. })
        ));
    }

    /// Helper: parse `script`, run to skip, and collect every visible
    /// `String` block on the rendered path. Lets req-gating tests make
    /// black-box assertions about which lines actually rendered.
    fn run_and_collect_strings(script: &str) -> Vec<String> {
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        runtime
            .current_blocks()
            .into_iter()
            .filter_map(|b| match b.block_type {
                BlockType::String(id) => Some(runtime.database.strings[id].clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn req_passing_shows_parent() {
        let script = "--- variables\nint x = 1\n---\nGated.\n  req x = 1\nAfter.";
        assert_eq!(run_and_collect_strings(script), vec!["Gated.", "After."]);
    }

    #[test]
    fn req_failing_skips_parent() {
        let script = "--- variables\nint x = 0\n---\nGated.\n  req x = 1\nAfter.";
        assert_eq!(run_and_collect_strings(script), vec!["After."]);
    }

    #[test]
    fn req_failing_skips_descendants() {
        let script =
            "--- variables\nint x = 0\n---\nParent.\n  req x = 1\n  Child.\n  Other child.\nAfter.";
        assert_eq!(run_and_collect_strings(script), vec!["After."]);
    }

    #[test]
    fn req_multiple_siblings_act_as_and() {
        let script = "--- variables\nint x = 5\n---\n\
            Both pass.\n  req x > 0\n  req x < 10\n\
            One fails.\n  req x > 0\n  req x > 100\n\
            All three pass.\n  req x >= 5\n  req x <= 5\n  req x != 0\n";
        assert_eq!(
            run_and_collect_strings(script),
            vec!["Both pass.", "All three pass."]
        );
    }

    #[test]
    fn req_short_circuits_failing_parent() {
        // The inner `req inner = 10 / inner` would div-by-zero if
        // evaluated. The outer parent's failing `req` must skip the
        // entire subtree without ever entering the inner block — so the
        // inner `req` is never evaluated and no runtime error fires.
        let script = "--- variables\nint outer = 0\nint inner = 0\n---\n\
            Outer fails.\n  req outer = 1\n  Never shown.\n    req inner = 10 / inner\n\
            After.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(runtime.take_last_error().is_none());
        let visible: Vec<String> = runtime
            .current_blocks()
            .into_iter()
            .filter_map(|b| match b.block_type {
                BlockType::String(id) => Some(runtime.database.strings[id].clone()),
                _ => None,
            })
            .collect();
        assert_eq!(visible, vec!["After."]);
    }

    #[test]
    fn req_set_flips_outcome() {
        // `set` mutates the variable mid-script, flipping which `req`
        // passes from "before" to "after".
        let script = "--- variables\nint flag = 0\n---\n\
            Before zero.\n  req flag = 0\n\
            Before one.\n  req flag = 1\n\
            set flag = 1\n\
            After zero.\n  req flag = 0\n\
            After one.\n  req flag = 1\n";
        assert_eq!(
            run_and_collect_strings(script),
            vec!["Before zero.", "After one."]
        );
    }

    #[test]
    fn req_division_by_zero_at_runtime() {
        let script = "--- variables\nint x = 0\nint y = 0\n---\n\
            Gated.\n  req x = 10 / y\n\
            After.";
        let (database, _warnings) = cuentitos_parser::parse(script).unwrap();
        let mut runtime = Runtime::new(database);
        runtime.run();
        runtime.skip();
        assert!(matches!(
            runtime.take_last_error(),
            Some(RuntimeError::DivisionByZero { .. })
        ));
    }

    #[test]
    fn req_negative_literal_rhs_compares_correctly() {
        let script = "--- variables\nint balance = -5\n---\n\
            Above bound.\n  req balance > -10\n\
            Equals minus five.\n  req balance = -5\n\
            Above zero.\n  req balance > 0\n";
        assert_eq!(
            run_and_collect_strings(script),
            vec!["Above bound.", "Equals minus five."]
        );
    }

    #[test]
    fn variable_type_mismatch_error_displays_and_is_constructible() {
        // The `VariableTypeMismatch` branch of `set_variable_value` is
        // unreachable while `Value` has a single variant — every
        // value trivially shares its kind with the declared one. This test
        // pins the user-facing error contract (variant exists, carries the
        // variable name, and renders a clear message) so the wiring is
        // verified today and the branch becomes triggerable as soon as a
        // second `Value` variant is introduced.
        let err = RuntimeError::VariableTypeMismatch {
            name: "x".to_string(),
        };
        let rendered = format!("{}", err);
        assert!(
            rendered.contains("Type mismatch"),
            "unexpected message: {}",
            rendered
        );
        assert!(rendered.contains("'x'"), "unexpected message: {}", rendered);
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
            include_str!("../../compatibility-tests/strings/feature/two-lines-and-end.md"),
            "two-lines-and-end.md",
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
            include_str!("../../compatibility-tests/strings/feature/two-lines-and-skip.md"),
            "two-lines-and-skip.md",
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
            include_str!("../../compatibility-tests/strings/feature/two-lines-and-end.md"),
            "two-lines-and-end.md",
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
            include_str!("../../compatibility-tests/strings/feature/two-lines-and-skip.md"),
            "two-lines-and-skip.md",
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
            include_str!("../../compatibility-tests/strings/feature/two-lines-and-skip.md"),
            "two-lines-and-skip.md",
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
            include_str!("../../compatibility-tests/strings/feature/two-lines-and-end.md"),
            "two-lines-and-end.md",
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

        assert!(runtime.running());

        runtime.stop();

        assert!(!runtime.running());
    }

    #[test]
    fn test_nested_block_traversal() {
        let test_case = TestCase::from_string(
            include_str!(
                "../../compatibility-tests/strings/feature/nested-strings-with-siblings.md"
            ),
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
