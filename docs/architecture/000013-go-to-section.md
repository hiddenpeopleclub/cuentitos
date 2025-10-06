# Go To Section Navigation

### Submitters

- Claude Code (with Fran Tufro)

## Change Log

- [accepted] 2025-10-06 - Accepted and implemented
- [draft] 2025-10-06 - Initial draft for go-to-section feature

## Referenced Use Case(s)

- [Basic Jump to Sibling](../../compatibility-tests/00000000032-basic-jump-to-sibling.md)
- [Jump Using Absolute Path](../../compatibility-tests/00000000033-jump-absolute-path.md)
- [Jump Using Relative Child Path](../../compatibility-tests/00000000036-jump-relative-child.md)
- [Jump Using Relative Sibling Path](../../compatibility-tests/00000000037-jump-relative-sibling.md)
- [Jump to Parent Using ..](../../compatibility-tests/00000000038-jump-to-parent.md)
- [Restart Current Section Using .](../../compatibility-tests/00000000039-restart-current-section.md)
- [Jump Using Combined Path](../../compatibility-tests/00000000040-jump-parent-then-sibling.md)
- [Section Not Found Error](../../compatibility-tests/00000000052-section-not-found-absolute.md)
- [Unreachable Code Warning](../../compatibility-tests/00000000063-warning-unreachable-siblings.md)

## Context

Interactive narratives often require non-linear navigation - jumping to different story sections based on choices, conditions, or narrative structure. While sections (ADR 000011) provide organizational structure, there was no mechanism to actually navigate between them. This ADR introduces the `->` syntax to enable explicit jumps to any section in the narrative tree.

This feature enables:
- Choice-based branching ("Go to Chapter 2")
- Loop structures (restart a section)
- Skip-ahead mechanics (jump to epilogue)
- Conditional navigation (future: "if condition -> section")

## Proposed Design

### Syntax

Section jumps use arrow syntax with one space after the arrow:
```cuentitos
# Section A
Text in A
-> Section B

# Section B
Text in B
```

### Path Types

1. **Absolute paths** (with backslash separators):
   ```cuentitos
   -> Root Section \ Child Section
   ```

2. **Relative paths** (simple names):
   ```cuentitos
   # Parent
     ## Child A
     -> Child B    // Finds sibling Child B
     ## Child B
     Text
   ```

3. **Parent navigation** (using `..`):
   ```cuentitos
   # Parent
     ## Child
     -> ..    // Jump back to Parent
   ```

4. **Current section restart** (using `.`):
   ```cuentitos
   # Loop
   Text
   -> .    // Restart Loop section
   ```

5. **Combined paths** (mixing `..` with names):
   ```cuentitos
   -> .. \ Sibling Section
   -> .. \ .. \ Uncle Section
   ```

### Relative Path Resolution

When resolving a relative path (e.g., `-> Target`), the system searches in this order:
1. **Children** of the current section
2. **Siblings** of the current section

Children are always preferred over siblings. To reference parent sections, use `..` explicitly.

### Syntax Validation Rules

1. **Spacing**: At least one space after `->` (extra spaces trigger warning), exactly one space around `\` in paths
2. **Section names**: Cannot contain `\` character
3. **Section names**: Cannot have leading/trailing whitespace (trimmed with warning)
4. **Goto paths**: Leading/trailing whitespace in paths is allowed but triggers warning (lenient for usability)
5. **Empty references**: `->` alone or `-> \` are parse errors
6. **Indentation**: The `->` command follows the same indentation rules as text

### Core Implementation

#### 1. BlockType Extension

Add a new variant to `BlockType`:
```rust
pub enum BlockType {
    Start,
    String(StringId),
    Section { id: String, display_name: String },
    GoToSection {
        path: String,           // Original path for debugging
        target_block_id: BlockId // Resolved at compile-time
    },
    End,
}
```

#### 2. Parser Layer

Create `go_to_section_parser.rs` implementing `FeatureParser`:
- Parse `-> path` syntax
- Validate spacing rules
- Parse path segments (handle `..`, `.`, section names)
- Return parsed path as structured data

Integrate into `parser.rs` main loop:
- After trying `section_parser`, before `line_parser`
- Create GoToSection block with placeholder target_block_id (0)
- Store original path string

#### 3. Compile-Time Validation Pass

After parsing completes, before returning Database:

**Step 1: Build Section Registry**
- Map section paths to BlockIds
- Track section hierarchy (parent-child-sibling relationships)
- Use display names as keys

**Step 2: Validate Each GoToSection Block**
- Find containing section (walk up parents to nearest Section block)
- Resolve path based on context:
  - Absolute paths: Look up in registry directly
  - Relative paths: Search children first, then siblings
  - `..` paths: Navigate up section hierarchy
  - `.` paths: Reference containing section
- Validate section exists (error if not found)
- Validate no navigation above root (error if `..` goes too far)
- Update GoToSection block with resolved target_block_id

**Step 3: Additional Validations**
- Detect empty sections (sections with no content blocks)
- Detect unreachable blocks after GoToSection (warn for all siblings and children)
- Validate section names don't contain `\` (parse error)
- Warn on section names with leading/trailing whitespace (trim and warn)

**Step 4: Collect Errors and Warnings**
- Use existing error collection pattern from Parser
- Return all errors together for better developer experience
- Warnings are printed but don't fail compilation

#### 4. Runtime Layer

Modify `Runtime::find_next_block()`:
```rust
fn find_next_block(&self) -> Option<usize> {
    let current_block = &self.database.blocks[self.program_counter];

    // Check if current block is a GoToSection
    if let BlockType::GoToSection { target_block_id, .. } = current_block.block_type {
        return Some(target_block_id);
    }

    // ... existing traversal logic
}
```

The runtime simply uses the pre-resolved target_block_id, making execution trivial.

### Error Messages

1. **Section not found**: `"Section not found: {path}"`
2. **Malformed syntax**: `"Expected section name after '->'"`
3. **Navigation above root**: `"Cannot navigate above root level"`
4. **Section name with backslash**: `"Section names cannot contain '\' character: {name}"`
5. **Empty section**: `"Section must contain at least one block: {name}"`
6. **Unreachable code**: `"Unreachable code after section jump"`
7. **Whitespace warning**: `"Section name has leading/trailing whitespace: '{original}'. Trimmed to '{trimmed}'"`

### Implementation Trade-offs

**Option A: Store path string, resolve at runtime**
- ❌ Runtime failures possible
- ❌ No compile-time guarantees

**Option B: Store path string, validate at compile-time, resolve at runtime**
- ✅ Errors caught early with line numbers
- ❌ Runtime must resolve paths every time

**Option C: Resolve and store BlockId at compile-time** (CHOSEN)
- ✅ All errors caught at compile-time with line numbers
- ✅ Fastest runtime (no path resolution needed)
- ✅ Simplest runtime logic
- ✅ Aligns with "fail early" philosophy
- ❌ More complex compile-time validation logic (acceptable trade-off)

## Considerations

### Infinite Loops

The feature allows creating infinite loops:
```cuentitos
# Section A
-> Section B

# Section B
-> Section A
```

**Decision**: Do not detect or prevent infinite loops. This is intentional behavior that may be desired (e.g., game loops). Users can always quit with the `q` command.

### Self-References

Sections can jump to themselves:
```cuentitos
# Loop
Text
-> Loop
```

**Decision**: Allow self-references. This enables loop constructs and is not inherently problematic.

### Unreachable Code Detection

Code after a `->` command is unreachable:
```cuentitos
# Section A
-> Section B
This text is unreachable
```

**Decision**: Warn but allow. This gives users feedback but doesn't prevent valid use cases (e.g., temporarily commenting out navigation during development).

### Case Sensitivity

Section names are case-sensitive:
```cuentitos
# Section A
-> section a    // Does NOT match "Section A"
```

**Decision**: Case-sensitive matching. This is consistent with most programming languages and prevents ambiguity.

### Interaction with Comments

Comments can appear before/after `->` commands:
```cuentitos
# Section A
// This jumps to B
-> Section B
// This comment is unreachable
```

**Decision**: Allow comments anywhere. The unreachable code warning applies to comments after jumps as well.

### Tree Traversal After Jump

After jumping to a section and executing its content, normal tree traversal continues:
```cuentitos
# Section A
-> Section C

# Section B
Skipped

# Section C
Executed

# Section D
Also executed (because it follows C in tree order)
```

**Decision**: Continue normal traversal after the jumped section. This makes behavior predictable and composable.

## Decision

Implement **Option C**: Full compile-time resolution with pre-computed BlockIds stored in GoToSection blocks.

This approach:
- Catches all errors early with clear line number information
- Makes runtime execution trivial and fast
- Aligns with the project's philosophy of early error detection
- Provides the best developer experience

### Implementation Steps

1. Add `GoToSection` variant to `BlockType` enum
2. Create `go_to_section_parser.rs` with `FeatureParser` trait
3. Integrate parser into main parse loop
4. Implement compile-time validation pass:
   - Build section registry
   - Resolve all GoToSection references
   - Validate paths and detect errors
   - Detect unreachable code
   - Validate section names and empty sections
5. Update runtime's `find_next_block()` to handle GoToSection
6. Add comprehensive unit tests for parser and validator (39 parser tests, 16 runtime tests)
7. Create 36 compatibility tests for go-to-section feature (60 total tests in suite)

## Consequences

### Positive Outcomes

**1. Full Compile-Time Validation Achieved**
- All path resolution happens at parse time with pre-computed BlockIds
- Runtime execution is trivial (just jump to target_block_id)
- All errors include accurate line numbers from the source file
- Developers get immediate feedback on navigation errors

**2. Comprehensive Test Coverage**
- 36 compatibility tests covering all navigation patterns
- 39 parser unit tests for validation and error handling
- 16 runtime tests including infinite loop verification
- 100% of compatibility tests passing

**3. Clean Architecture**
- `go_to_section_parser.rs` implements FeatureParser trait cleanly
- Validation logic centralized in `validate_and_resolve()` method
- No changes needed to Block tree structure or runtime traversal
- Warning system provides helpful feedback without blocking compilation

**4. Developer Experience**
- Lenient whitespace handling with warnings (not errors) improves usability
- Multiple error reporting shows all issues at once
- Clear, actionable error messages with file and line information
- Unreachable code warnings help catch logical errors

### Performance Considerations

**Initial Implementation**
- O(n²) complexity in path resolution due to unnecessary cloning
- Fixed by iterating with references instead of range-based indexing

**Current Performance**
- Parse-time resolution adds minimal overhead (single pass after parsing)
- Runtime has zero path resolution overhead
- Section registry built once during validation
- Overall performance impact: negligible for typical scripts

### Lessons Learned

**1. Borrow Checker Challenges**
- Initial implementation had borrow checker issues when mutating blocks while iterating
- Solution: Collect block information first, then mutate in separate pass
- Pattern: `filter_map().collect()` followed by iteration over collected data

**2. Line Number Tracking**
- Initially used `block_id` as proxy for line numbers (incorrect)
- Fixed by using actual `block.line` field throughout
- Importance: Accurate line numbers critical for developer experience

**3. Code Duplication**
- Path building logic initially duplicated in two methods
- Consolidated into single `build_section_path_string()` method
- Takeaway: Watch for similar traversal patterns that can be unified

**4. Whitespace Philosophy**
- Section definitions: Strict (error on invalid formatting)
- Goto commands: Lenient (warn and trim)
- Rationale: Section definitions are structural, goto paths are usage
- This balance improves usability without sacrificing correctness

**5. Test-Driven Development Effectiveness**
- Writing compatibility tests first defined behavior precisely
- Bug fixes followed pattern: unit test → fix → verify compatibility test
- TDD caught edge cases early (empty sections, unreachable code, etc.)

### Code Quality Improvements

Post-implementation code review identified and fixed:
- Removed O(n²) performance issue in validation loop
- Fixed line number tracking consistency
- Eliminated code duplication in path building
- Improved test assertions (removed clippy warnings)
- Added comprehensive documentation to complex algorithms

### Impact on Codebase

**Added:**
- `GoToSection` variant to `BlockType` enum
- `go_to_section_parser.rs` (210 lines)
- Path resolution logic in `parser.rs` (~300 lines)
- Validation and warning system
- 36 compatibility tests
- 39 parser unit tests with full coverage

**Modified:**
- `Runtime::find_next_block()` to handle GoToSection (3 lines)
- CLI output to display "QUIT" command
- Parser main loop to integrate go-to-section parser

**No Changes Needed:**
- Block tree structure
- Database schema
- String management
- Indentation system
- Comment support

### Future Considerations

**Potential Enhancements:**
- Circular reference detection (currently allowed intentionally)
- Compile-time infinite loop warnings (optional, low priority)
- Path auto-completion hints in errors (suggest similar section names)
- "Strict mode" where warnings become errors

**Maintenance Notes:**
- Path resolution is well-documented and tested
- Adding new navigation operators (e.g., `~` for root) would follow same pattern
- Validation logic is isolated and easy to extend
- Test suite provides regression protection

## Other Related ADRs

- [Sections and Navigation Support](000011-sections-and-navigation.md) - Foundation for section structure
- [Modular Parser Architecture](000010-modular-parser-architecture.md) - FeatureParser trait pattern
- [Indentation and Block Parenting](000009-indentation-block-parenting.md) - Hierarchical structure basis
- [Comment Support](000012-comment-support.md) - Interaction with comments

## References

- [Goto statement](https://en.wikipedia.org/wiki/Goto) - Historical context for jump commands
- [Twine passage navigation](https://twinery.org/) - Similar narrative navigation in interactive fiction
- [Ink divert syntax](https://github.com/inkle/ink/blob/master/Documentation/WritingWithInk.md#diverts) - Similar `->` syntax in another narrative language
