# Go To Commands in CLI

### Submitters

- Claude Code (with Fran Tufro)

## Change Log

- [draft] 2025-10-08 - Initial draft for CLI GoTo feature

## Referenced Use Case(s)

- [CLI GoTo Section](../../compatibility-tests/00000000115-cli-goto-section.md)
- [CLI Call and Return](../../compatibility-tests/00000000116-cli-call-and-return.md)
- [CLI GoTo END](../../compatibility-tests/00000000117-cli-goto-end.md)
- [CLI GoTo START](../../compatibility-tests/00000000118-cli-goto-start.md)
- [CLI GoTo RESTART](../../compatibility-tests/00000000119-cli-goto-restart.md)
- [Mix Script and CLI GoTos](../../compatibility-tests/00000000120-cli-mix-script-and-cli-gotos.md)
- [CLI GoTo with Relative Child Path](../../compatibility-tests/00000000121-cli-goto-relative-child.md)
- [CLI GoTo with Parent Navigation](../../compatibility-tests/00000000122-cli-goto-parent-navigation.md)
- [CLI GoTo Current Section](../../compatibility-tests/00000000123-cli-goto-current-section.md)
- [CLI GoTo with Absolute Path](../../compatibility-tests/00000000124-cli-goto-absolute-path.md)
- [CLI GoTo During Call Stack](../../compatibility-tests/00000000125-cli-goto-during-call-stack.md)
- [CLI Call During Call Stack](../../compatibility-tests/00000000126-cli-call-during-call-stack.md)
- [Multiple CLI GoTos in Sequence](../../compatibility-tests/00000000127-cli-multiple-gotos-sequence.md)
- [CLI GoTo After Script GoTo](../../compatibility-tests/00000000128-cli-goto-after-script-goto.md)
- [CLI Error: Section Not Found](../../compatibility-tests/00000000129-cli-error-section-not-found.md)
- [CLI Error: Navigate Above Root](../../compatibility-tests/00000000130-cli-error-navigate-above-root.md)
- [CLI Error: Malformed GoTo Empty](../../compatibility-tests/00000000131-cli-error-malformed-empty.md)
- [CLI Error: Trailing Backslash](../../compatibility-tests/00000000132-cli-error-trailing-backslash.md)
- [CLI Error: No Space After Arrow](../../compatibility-tests/00000000133-cli-error-no-space-after-arrow.md)

## Context

While the cuentitos language supports GoTo commands within scripts (ADR 000013, 000014, 000015), there was no way for users to navigate during runtime via the CLI. This limitation meant users could only use predefined navigation commands (`n` for next, `s` for skip, `q` for quit) and couldn't explore alternative narrative paths interactively.

This ADR enables users to type GoTo commands directly in the CLI during runtime, allowing:
- Interactive exploration of narrative branches
- Testing different story paths without editing scripts
- Debugging navigation logic
- Creative play and experimentation with narrative structure

## Proposed Design

### CLI Input Syntax

Users can type GoTo commands using the exact same syntax as scripts:

```bash
# Permanent jumps
-> Section A
-> Parent \ Child
-> ..
-> .
-> END
-> START
-> RESTART

# Call and return
<-> Section B
<-> Parent \ Child \ Grandchild
<-> ..
```

### Architecture

#### 1. Shared Path Resolution Module (`common/src/path_resolver.rs`)

The path resolution logic used by the parser during compilation needs to be available at runtime. We'll extract this into a shared module:

```rust
pub struct PathResolver<'a> {
    database: &'a Database,
    section_registry: &'a HashMap<String, SectionId>,
}

impl<'a> PathResolver<'a> {
    pub fn resolve_path(
        &self,
        path: &str,
        containing_section: Option<BlockId>,
    ) -> Result<ResolvedPath, PathResolutionError> {
        // Handles:
        // - Special keywords: START, RESTART, END
        // - Current section: .
        // - Parent navigation: ..
        // - Absolute paths: Root \ Child
        // - Relative paths: Child, Sibling
    }
}

pub enum ResolvedPath {
    Section(SectionId),
    Start,
    Restart,
    End,
}
```

**Rationale:** Both parser and runtime need identical path resolution logic. Extracting to `common/` ensures:
- Single source of truth for path semantics
- No code duplication
- Consistent behavior between compile-time and runtime

#### 2. Runtime Error Types (`runtime/src/error.rs`)

New error enum for runtime-specific errors:

```rust
#[derive(Debug, Clone)]
pub enum RuntimeError {
    SectionNotFound { path: String },
    NavigationAboveRoot,
    InvalidPath { message: String },
    NotRunning,
}

impl fmt::Display for RuntimeError {
    // User-friendly error messages for CLI display
}
```

**Rationale:** Separate from `ParseError` because:
- Different error context (runtime vs. compile-time)
- No file/line information at runtime
- User-facing error messages (not developer errors)

#### 3. Runtime GoTo Methods (`runtime/src/lib.rs`)

The runtime exposes methods that encapsulate all state manipulation:

```rust
impl Runtime {
    // Enhanced path resolution using shared module
    pub fn find_section_by_path(&self, path: &str) -> Result<ResolvedPath, RuntimeError>

    // GoTo methods - manipulate state directly
    pub fn goto_section(&mut self, section_id: SectionId) -> Result<(), RuntimeError>
    pub fn goto_and_back_section(&mut self, section_id: SectionId) -> Result<(), RuntimeError>
    pub fn goto_start(&mut self) -> Result<(), RuntimeError>
    pub fn goto_restart(&mut self) -> Result<(), RuntimeError>
    pub fn goto_end(&mut self) -> Result<(), RuntimeError>
}
```

**State Manipulation:**
- `goto_section`: Set `program_counter` to section's block_id, add to `current_path`
- `goto_and_back_section`: Push to `call_stack`, set `program_counter`, add to `current_path`
- `goto_start`: Clear `call_stack`, jump to block 0
- `goto_restart`: Clear `call_stack`, reset state, jump to block 0
- `goto_end`: Jump to last block (END)

**Rationale:** Encapsulating state manipulation in methods:
- Keeps state management logic in runtime (not CLI)
- Ensures correct behavior matches script execution
- Makes CLI simple and focused on I/O

#### 4. CLI Changes (`cli/src/main.rs`)

Update `process_input()` to handle GoTo commands:

```rust
fn process_input(input: &str, runtime: &mut Runtime) -> bool {
    let trimmed = input.trim();

    // Check for GoTo commands
    if trimmed.starts_with("<->") {
        return handle_goto_and_back(trimmed, runtime);
    } else if trimmed.starts_with("->") {
        return handle_goto(trimmed, runtime);
    }

    // Existing commands (n, s, q)
    match trimmed {
        "n" => { /* ... */ }
        "s" => { /* ... */ }
        "q" => false,
        _ => {
            eprintln!("Unknown command: {}", trimmed);
            false
        }
    }
}

fn handle_goto(input: &str, runtime: &mut Runtime) -> bool {
    // Parse: extract path from "-> Section A"
    // Call runtime.find_section_by_path(path)
    // Call appropriate goto method based on ResolvedPath
    // Display error or continue
}

fn handle_goto_and_back(input: &str, runtime: &mut Runtime) -> bool {
    // Same as handle_goto but uses goto_and_back_section
}
```

**Input Validation:**
- Must have space after `->` or `<->`
- Path cannot be empty or end with trailing `\`
- Whitespace is trimmed (silently, unlike parser warnings)

**Rationale:** CLI only handles I/O and delegates logic to runtime:
- Parsing is simple (split on `->` or `<->`)
- Error handling displays and continues (doesn't exit)
- Rendering uses existing `render_current_blocks()`

#### 5. Section Registry

**Decision:** Store section registry in Database during parsing.

```rust
// In Database
pub struct Database {
    pub blocks: Vec<Block>,
    pub strings: Vec<String>,
    pub sections: Vec<Section>,
    pub section_registry: HashMap<String, SectionId>, // NEW
}
```

The parser builds this registry during the validation pass and stores it in the database. Runtime can then use it directly without rebuilding.

**Rationale:**
- Registry is built once during parsing (no runtime overhead)
- Database is the source of truth for all script data
- Runtime doesn't need to traverse the tree to build registry
- Consistent with how sections and strings are stored

### Behavior

#### Exact Script Equivalence

CLI GoTo commands behave **exactly** as if they appeared in the script:

```cuentitos
# In script
Text before
-> Section B
Text after (unreachable)
```

is equivalent to user typing:

```bash
Input: n
Input: -> Section B
```

Same state transitions, same rendering, same errors.

#### Error Handling

Errors display to stderr and **do not change runtime state**:

```
Input: -> Fake Section
Output: ERROR: Section not found: Fake Section
[Runtime state unchanged, waits for next input]
```

This allows users to recover from typos and continue exploration.

#### Context for Relative Paths

Relative path resolution uses the current program counter's context:

```cuentitos
# Parent
  ## Child A
    Text in A    <- [Program counter here]
  ## Child B
```

If user types `-> ..`, it resolves to `Parent` based on current position.

## Considered Alternatives

### Alternative 1: Build Registry at Runtime

Instead of storing in Database, runtime could build registry on demand.

**Rejected because:**
- Duplicates work (parser already builds it)
- Adds startup cost to runtime
- Database should be the complete source of truth

### Alternative 2: Special CLI Syntax

Use different syntax for CLI (e.g., `jump Section A` instead of `-> Section A`).

**Rejected because:**
- Adds cognitive load (learn two syntaxes)
- Violates principle of script equivalence
- Makes testing harder (different behavior)

### Alternative 3: Duplicate Path Resolution in Runtime

Copy path resolution logic from parser to runtime instead of sharing.

**Rejected because:**
- Code duplication is error-prone
- Semantic drift between parser and runtime
- Harder to maintain and test

## Consequences

### Positive

- **Interactive exploration:** Users can experiment with narrative branches without editing scripts
- **Better debugging:** Developers can test navigation logic interactively
- **Code reuse:** Path resolution logic is shared between parser and runtime
- **Consistent behavior:** CLI GoTo behaves exactly like script GoTo
- **Extensibility:** Foundation for future features (conditional gotos, dynamic navigation)

### Negative

- **Increased surface area:** More commands to test and maintain
- **Error complexity:** Runtime errors need good UX (clear messages, graceful handling)
- **State management:** GoTo methods must carefully maintain runtime state consistency

### Neutral

- **Database size:** Section registry adds to database size (negligible for most scripts)
- **Parser changes:** Refactoring to use shared path resolver requires touching parser code

## Implementation Notes

1. **Order of implementation:**
   - Create `PathResolver` in `common/`
   - Create `RuntimeError` in `runtime/`
   - Add section registry to `Database`
   - Update parser to use shared `PathResolver` and populate registry
   - Enhance runtime with goto methods
   - Update CLI to parse and handle goto input

2. **Testing strategy:**
   - 19 compatibility tests cover all scenarios
   - Unit tests for `PathResolver` in `common/`
   - Unit tests for runtime goto methods
   - Error message formatting tests

3. **Error messages:**
   - Keep consistent with parser errors where applicable
   - Use simple, action-oriented language
   - No file/line information (not available at runtime)

## Future Considerations

- **Goto history:** Track visited sections for "back" command
- **Bookmarks:** Save positions to return to later
- **Conditional gotos:** `if var > 5 -> Section A` (when variables exist)
- **Goto macros:** User-defined shortcuts for complex paths
