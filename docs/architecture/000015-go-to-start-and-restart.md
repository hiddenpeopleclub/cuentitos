# Go To Start and Restart

### Submitters

- Claude Code (with Fran Tufro)

## Change Log

- [draft] 2025-01-10 - Initial draft for go-to-start and restart feature

## Referenced Use Case(s)

- [Jump to START from Root Level](../../compatibility-tests/00000000102-goto-start-from-root.md)
- [Jump to START from Within Subsection](../../compatibility-tests/00000000103-goto-start-from-subsection.md)
- [Jump to START Shows All Blocks](../../compatibility-tests/00000000104-goto-start-shows-all-blocks.md)
- [Jump to RESTART from Root Level](../../compatibility-tests/00000000105-goto-restart-from-root.md)
- [Jump to RESTART from Within Subsection](../../compatibility-tests/00000000106-goto-restart-from-subsection.md)
- [Call to START with Warning](../../compatibility-tests/00000000107-call-start-warning.md)
- [Call to RESTART with Warning](../../compatibility-tests/00000000108-call-restart-warning.md)
- [Error: Section Named START](../../compatibility-tests/00000000109-error-section-named-start.md)
- [Error: Section Named RESTART](../../compatibility-tests/00000000110-error-section-named-restart.md)
- [Error: Subsection Named START](../../compatibility-tests/00000000111-error-subsection-named-start.md)
- [Error: Subsection Named RESTART](../../compatibility-tests/00000000112-error-subsection-named-restart.md)
- [RESTART Clears Call Stack](../../compatibility-tests/00000000113-restart-clears-call-stack.md)
- [START Clears Call Stack](../../compatibility-tests/00000000114-start-preserves-call-stack.md)

## Context

Interactive narratives often need to restart from the beginning, either to replay content or to reset state. While `-> END` (ADR 000013) allows jumping to the end, there was no symmetric way to jump back to the start. Additionally, when variables are introduced, narratives will need a way to reset all state and truly restart from a clean slate.

This ADR introduces two new navigation commands:
- `-> START`: Jump back to the beginning of the narrative
- `-> RESTART`: Clear all runtime state and jump to the beginning

Use cases include:
- Replay loops (play again after completion)
- Tutorial restart mechanisms
- Menu systems that return to start
- State reset for testing different paths
- Game over → restart functionality

## Proposed Design

### Syntax

Both commands use the same arrow syntax as other goto commands:

```cuentitos
# Section A
Text in A
-> START

# Section B
Text in B
-> RESTART
```

### Behavior

**`-> START`**: Jump to the START block (block 0)
- Clears the call stack (won't return from `<->` calls)
- Preserves execution path history
- Does NOT reset other runtime state (future: preserves variables)

**`-> RESTART`**: Clear all state and jump to START block
- Calls `runtime.reset()` which resets all runtime state
- Clears call stack, execution path, program counters
- Future: will clear variables, random seeds, etc.
- Provides a truly clean restart

### Reserved Keywords

`START` and `RESTART` are reserved section names (like `END`):

```cuentitos
# START        // ERROR: Section name "START" is reserved
# RESTART      // ERROR: Section name "RESTART" is reserved
```

### Warnings for Call-and-Return

Using `<->` with START or RESTART produces warnings (they won't return):

```cuentitos
<-> START     // WARNING: will not return (restarts from beginning)
<-> RESTART   // WARNING: will not return (clears state and restarts from beginning)
```

## Core Implementation

### 1. BlockType Refactoring

**Problem**: Currently, goto commands store both `path: String` and `target_block_id: BlockId`. This creates redundancy and relies on string matching at various points.

**Solution**: Introduce explicit BlockType variants for each goto type, removing the need for string matching and making the type system more expressive.

#### New BlockType Structure

```rust
pub type SectionId = BlockId;  // Type alias for future extensibility

pub enum BlockType {
    Start,
    String(StringId),
    Section {
        id: String,
        display_name: String,
    },
    GoTo(SectionId),           // Regular section jump (was GoToSection)
    GoToAndBack(SectionId),    // Call and return (was GoToSectionAndBack)
    GoToStart,                 // Jump to START block
    GoToRestart,               // Clear state + jump to START
    GoToEnd,                   // Jump to END block
    End,
}
```

**Key Changes**:
- Remove `path` field from goto blocks (can be derived from SectionId when needed)
- Use `SectionId` type alias (will map to section metadata in future)
- Explicit variants for special gotos (Start, Restart, End)
- Clearer naming: `GoTo` instead of `GoToSection`, `GoToAndBack` instead of `GoToSectionAndBack`

**Benefits**:
- Type-safe: No string matching needed
- Future-proof: SectionId will eventually map to section metadata
- Explicit: Each goto type is clearly distinguished
- Cleaner: No redundant path storage

### 2. Parser Changes

#### a) Modify `resolve_path()` Method

Change signature to return the appropriate BlockType variant:

```rust
fn resolve_path(
    &self,
    path: &str,
    containing_section: Option<BlockId>,
    registry: &HashMap<String, (BlockId, usize)>,
    database: &Database,
    line: usize,
) -> Result<BlockType, ParseError>
```

Handle special keywords:
```rust
let path = path.trim();

match path {
    "START" => return Ok(BlockType::GoToStart),
    "RESTART" => return Ok(BlockType::GoToRestart),
    "END" => return Ok(BlockType::GoToEnd),
    // ... rest of path resolution for regular sections
}
```

#### b) Update `validate_section_names()` Method

Add START and RESTART to reserved words:

```rust
fn validate_section_names(&mut self, database: &Database) -> Result<(), ParseError> {
    for block in database.blocks.iter() {
        if let BlockType::Section { display_name, .. } = &block.block_type {
            match display_name.as_str() {
                "END" => {
                    self.errors.push(ParseError::InvalidSectionName {
                        message: "Section name \"END\" is reserved".to_string(),
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }
                "START" => {
                    self.errors.push(ParseError::InvalidSectionName {
                        message: "Section name \"START\" is reserved".to_string(),
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }
                "RESTART" => {
                    self.errors.push(ParseError::InvalidSectionName {
                        message: "Section name \"RESTART\" is reserved".to_string(),
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }
                _ => {}
            }
            // ... rest of validation
        }
    }
    Ok(())
}
```

#### c) Update `resolve_goto_sections()` Method

Generate warnings for `<->` with special targets:

```rust
if is_call_and_back {
    // Check if calling a special target that won't return
    let warning_message = match &target_block_type {
        BlockType::GoToStart => Some("<-> START will not return (restarts from beginning)"),
        BlockType::GoToRestart => Some("<-> RESTART will not return (clears state and restarts from beginning)"),
        BlockType::GoToEnd => Some("<-> END will not return (just end execution)"),
        _ => None,
    };

    if let Some(message) = warning_message {
        self.warnings.push(Warning {
            message: message.to_string(),
            file: self.file_path.clone(),
            line,
        });
    }

    context.database.blocks[block_id].block_type = BlockType::GoToAndBack(/* ... */);
} else {
    context.database.blocks[block_id].block_type = BlockType::GoTo(/* ... */);
}
```

### 3. Runtime State Refactoring

**Problem**: Runtime state is scattered across multiple fields, making it hard to reset cleanly.

**Solution**: Group all resettable state into a `RuntimeState` struct.

#### New Runtime Structure

```rust
#[derive(Debug, Clone)]
struct RuntimeState {
    program_counter: usize,
    previous_program_counter: usize,
    current_path: Vec<BlockId>,
    call_stack: Vec<CallFrame>,
    // Future: variables, random state, etc.
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
```

#### Add `reset()` Method

```rust
impl Runtime {
    pub fn reset(&mut self) {
        self.state = RuntimeState::with_start_block();
    }

    pub fn run(&mut self) {
        self.running = true;
        self.reset();
    }
}
```

### 4. Runtime Execution Changes

Modify `find_next_block()` to handle new BlockType variants:

```rust
fn find_next_block(&mut self) -> Option<usize> {
    if self.state.program_counter >= self.database.blocks.len() - 1 {
        return None;
    }

    let current_block = &self.database.blocks[self.state.program_counter];

    // Handle GoToAndBack variants
    if let BlockType::GoToAndBack(section_id) = current_block.block_type {
        // ... existing call stack logic
        return Some(section_id);
    }

    // Handle special GoToAndBack targets
    match &current_block.block_type {
        BlockType::GoToAndBack(section_id) => {
            // Existing logic for regular sections
            // ...
        }

        // Special targets that clear call stack and don't return
        BlockType::GoToStart => {
            self.state.call_stack.clear();
            return Some(0);  // START block
        }

        BlockType::GoToRestart => {
            self.reset();
            return Some(0);  // START block
        }

        BlockType::GoToEnd => {
            self.state.call_stack.clear();
            return Some(self.database.blocks.len() - 1);  // END block
        }

        _ => {}
    }

    // Handle GoTo variants
    match current_block.block_type {
        BlockType::GoTo(section_id) => Some(section_id),
        BlockType::GoToStart => {
            self.state.call_stack.clear();
            Some(0)
        }
        BlockType::GoToRestart => {
            self.reset();
            Some(0)
        }
        BlockType::GoToEnd => Some(self.database.blocks.len() - 1),
        _ => self.compute_natural_next_block(),
    }
}
```

## Implementation Trade-offs

### Option A: Keep `path` field in BlockType (Current Approach)

```rust
GoToSection {
    path: String,
    target_block_id: BlockId,
}
```

**Pros:**
- Useful for error messages at runtime
- Good for debugging

**Cons:**
- Redundant storage (path already resolved to BlockId)
- Relies on string matching for special cases (END)
- Can't easily distinguish START/RESTART/END from regular sections

### Option B: Remove `path`, add special BlockType variants (CHOSEN)

```rust
GoTo(SectionId)
GoToStart
GoToRestart
GoToEnd
```

**Pros:**
- Type-safe: compiler enforces correct handling
- No string matching needed
- Clear intent for each goto type
- Path can be derived from SectionId when needed for errors
- Future-proof for section metadata system

**Cons:**
- More BlockType variants
- Slightly more refactoring needed

### Option C: Use enum wrapper for target

```rust
enum GoToTarget {
    Section(SectionId),
    Start,
    Restart,
    End,
}

GoToSection { target: GoToTarget }
```

**Pros:**
- Groups related concepts
- Fewer top-level BlockType variants

**Cons:**
- Extra layer of indirection
- Less explicit in match statements
- Harder to pattern match on specific targets

**Decision**: Choose Option B for clarity and type safety.

## Considerations

### Call Stack Behavior

Both `-> START` and `-> RESTART` clear the call stack:

```cuentitos
# Section A
<-> Section B
This never executes

# Section B
-> START    // Clears call stack, won't return to A
```

**Rationale**: Jumping to START implies "start over", which shouldn't maintain the call stack from a previous execution context. This prevents confusing behavior where execution would resume in the middle of a previous flow.

### Difference Between START and RESTART

**`-> START`**:
- Minimal reset: only clears call stack
- Preserves execution history (current_path)
- Future: preserves variables

**`-> RESTART`**:
- Full reset: calls `reset()` method
- Clears all runtime state
- Future: resets variables, random seeds, counters

**Use Cases**:
- `-> START`: Menu systems, replay with memory of previous choices
- `-> RESTART`: True game restart, clean slate for testing

### Infinite Loops

Both commands can create infinite loops:

```cuentitos
Text line
-> START    // Infinite loop
```

**Decision**: Allow infinite loops (same as existing `-> .` behavior). Users can quit with `q` command.

### Future Extensibility

The `RuntimeState` struct makes it easy to add new state in the future:

```rust
struct RuntimeState {
    program_counter: usize,
    previous_program_counter: usize,
    current_path: Vec<BlockId>,
    call_stack: Vec<CallFrame>,
    variables: HashMap<String, Variable>,  // Future
    random_seed: u64,                      // Future
    visit_counts: HashMap<BlockId, u32>,  // Future
}
```

The `reset()` method will automatically handle all new state.

## Decision

Implement Option B: Explicit BlockType variants for each goto type, with RuntimeState refactoring.

This approach:
- Provides type safety and clarity
- Makes START/RESTART behavior explicit
- Sets up clean architecture for future state management
- Eliminates string matching in favor of type system
- Makes code more maintainable and extensible

### Implementation Steps

1. Add `SectionId` type alias to `common/src/block.rs`
2. Refactor `BlockType` enum with new variants
3. Fix compilation errors in parser (update all match statements)
4. Update `resolve_path()` to return appropriate BlockType variants
5. Add START/RESTART to reserved keyword validation
6. Add warnings for `<-> START` and `<-> RESTART`
7. Refactor Runtime to use `RuntimeState` struct
8. Implement `reset()` method
9. Update `find_next_block()` to handle new variants
10. Fix all unit tests in parser and runtime
11. Run compatibility tests to verify behavior
12. Update test 114 expectation (START clears call stack)

## Consequences

### Positive Outcomes

**1. Type-Safe Navigation System**
- Compiler enforces handling of all goto types
- No runtime string matching needed
- Clear distinction between regular and special jumps
- Future-proof for section metadata system

**2. Clean State Management**
- All resettable state in one place (`RuntimeState`)
- Easy to extend for variables and other state
- Clear semantics: `reset()` resets everything
- Foundation for save/load functionality

**3. Explicit Special Cases**
- START, RESTART, END are first-class BlockType variants
- No magic strings or special-case code paths
- Clear intent in both parser and runtime
- Better error messages possible (can reference specific goto type)

**4. Developer Experience**
- Reserved keyword validation prevents naming conflicts
- Warnings for `<->` misuse help catch logical errors
- Clear distinction between START (minimal) and RESTART (full)
- Consistent with existing `-> END` behavior

### Architectural Impact

**Changes Required:**
- BlockType enum refactoring (breaking change to common crate)
- Parser path resolution updated
- Runtime execution flow updated
- All match statements on BlockType need updating

**Benefits:**
- Eliminates technical debt (redundant path storage)
- Sets up cleaner architecture for future features
- Makes codebase more maintainable
- Reduces coupling between parser and runtime

### Performance Considerations

**Parse Time:**
- No change (still single-pass compilation)
- Reserved keyword validation adds negligible overhead

**Runtime:**
- No string matching needed (faster)
- Special gotos handled via match (zero overhead)
- `reset()` is O(1) for current state, O(n) future with variables (acceptable)

### Future Considerations

**Variable System Integration:**
When variables are added, `RuntimeState` will naturally extend:
```rust
struct RuntimeState {
    // ... existing fields
    variables: HashMap<String, Variable>,
}
```

And `reset()` will clear variables automatically.

**Save/Load System:**
The `RuntimeState` struct can be serialized for save/load:
```rust
pub fn save_state(&self) -> RuntimeState {
    self.state.clone()
}

pub fn load_state(&mut self, state: RuntimeState) {
    self.state = state;
}
```

**Section Metadata Database:**
`SectionId` will eventually map to:
```rust
struct SectionMetadata {
    block_id: BlockId,
    path: String,
    display_name: String,
    tags: Vec<String>,
}
```

The BlockType refactoring makes this future migration easier.

## Other Related ADRs

- [Go To Section](000013-go-to-section.md) - Foundation for navigation system
- [Go To Section and Back](000014-go-to-section-and-back.md) - Call stack mechanism
- [Sections and Navigation Support](000011-sections-and-navigation.md) - Section structure

## References

- [Ink restart](https://github.com/inkle/ink/blob/master/Documentation/WritingWithInk.md) - Similar restart mechanisms in narrative languages
- [Twine restart](https://twinery.org/) - Passage restart patterns
- Rust enum design patterns for type-safe variants
