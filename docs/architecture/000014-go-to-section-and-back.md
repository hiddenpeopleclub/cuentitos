# Go To Section and Back (Call and Return)

### Submitters

- Claude Code (with Fran Tufro)

## Change Log

- [draft] 2025-10-06 - Initial draft for go-to-section-and-back feature

## Referenced Use Case(s)

- [Basic Call and Return](../../compatibility-tests/00000000066-basic-call-and-return.md)
- [Nested Calls (Two Levels)](../../compatibility-tests/00000000069-nested-calls-two-levels.md)
- [Called Section Contains Jump](../../compatibility-tests/00000000076-called-section-contains-jump.md)
- [Skip Within Called Section](../../compatibility-tests/00000000079-skip-within-called-section.md)
- [Call to Section with Subsections](../../compatibility-tests/00000000082-call-to-section-with-subsections.md)

## Context

The existing `->` command (ADR 000013) enables one-way navigation between sections. Once you jump to a section, execution continues from there and never returns to the caller. This works for permanent narrative branches but doesn't support reusable narrative fragments or function-like behavior.

Interactive narratives often need to:
- **Call reusable sections** (e.g., a common dialogue or action sequence)
- **Return to the original flow** after executing a section
- **Compose narrative fragments** like building blocks
- **Create nested call structures** (A calls B, B calls C, returns to B, returns to A)

This ADR introduces the `<->` syntax to enable "call and return" semantics - jump to a section, execute it completely (including any jumps it makes), then return to continue from where it was called.

## Proposed Design

### Syntax

Section calls use double-arrow syntax with one space after the arrows:
```cuentitos
# Section A
Text in A
<-> Section B
Text after call in A

# Section B
Text in B
```

Output:
```
-> Section A
Text in A
-> Section B
Text in B
Text after call in A
```

### Key Behavior

**1. Call and Return:**
- `<->` jumps to a section and remembers the return point
- When the called section completes, execution returns and continues

**2. Code After Call is Reachable:**
Unlike `->`, code after `<->` executes (no unreachable code warning)

**3. Nesting Support:**
Calls can be nested arbitrarily deep, forming a call stack:
```cuentitos
# A
<-> B    // A calls B

# B
<-> C    // B calls C

# C
Text     // C returns to B, B returns to A
```

**4. Integration with Regular Jumps:**
If a called section contains `->` jumps, those are resolved before returning:
```cuentitos
# A
<-> B
Back in A

# B
-> C     // Jump to C

# C
Text     // After C finishes, return to A (not continue to D)
```

**5. Subsection Execution:**
Called sections execute all their subsections normally before returning

**6. Skip Behavior Changes:**
The `s` (skip) command now means "skip to end of current called section":
- If not in a call: skip to END (existing behavior)
- If in a call: skip to end of called section, then return
- Prevents infinite loops with recursive calls

**7. END Behavior:**
If execution reaches END during a call, the script terminates without returning

### Path Resolution

`<->` supports the same path types as `->`:
- Absolute paths: `<-> Root \ Child`
- Relative paths: `<-> Sibling`
- Parent navigation: `<-> ..`
- Current section: `<-> .` (recursive call)
- Combined paths: `<-> .. \ Sibling`

All path resolution logic is reused from the existing `GoToSection` implementation.

### Core Implementation

#### 1. BlockType Extension

Add new variant to `BlockType`:
```rust
pub enum BlockType {
    Start,
    String(StringId),
    Section { id: String, display_name: String },
    GoToSection {
        path: String,
        target_block_id: BlockId
    },
    GoToSectionAndBack {
        path: String,
        target_block_id: BlockId
    },
    End,
}
```

#### 2. Runtime Call Stack

Add call stack to track return points:
```rust
#[derive(Debug, Clone)]
struct CallFrame {
    return_block_id: BlockId,   // Block to return to after call completes
    called_section_id: BlockId, // The section that was called
}

pub struct Runtime {
    pub database: Database,
    running: bool,
    program_counter: usize,
    previous_program_counter: usize,
    current_path: Vec<BlockId>,
    call_stack: Vec<CallFrame>,  // NEW: Call stack for <-> commands
}
```

#### 3. Parser Layer

Create `go_to_section_and_back_parser.rs` implementing `FeatureParser`:
- Parse `<-> path` syntax (reuse logic from `go_to_section_parser.rs`)
- Validate spacing rules (same as `->`)
- Return parsed path as structured data

Integrate into `parser.rs` main loop:
- Check for `<->` before checking for `->`
- Create `GoToSectionAndBack` block with placeholder target_block_id
- Reuse existing compile-time validation and path resolution

#### 4. Modified find_next_block() Logic

```rust
fn find_next_block(&self) -> Option<usize> {
    let current_block = &self.database.blocks[self.program_counter];

    // Handle GoToSectionAndBack: push to call stack and jump
    if let BlockType::GoToSectionAndBack { target_block_id, .. } = current_block.block_type {
        // Find the return point (next block in normal traversal)
        let return_block_id = self.compute_natural_next_block()?;

        // Push call frame
        self.call_stack.push(CallFrame {
            return_block_id,
            called_section_id: target_block_id,
        });

        return Some(target_block_id);
    }

    // Handle GoToSection: just jump (existing logic)
    if let BlockType::GoToSection { target_block_id, .. } = current_block.block_type {
        return Some(target_block_id);
    }

    // Compute natural next block (children, siblings, parent's siblings)
    let natural_next = self.compute_natural_next_block()?;

    // Check if we should return from a call
    if let Some(frame) = self.call_stack.last() {
        // If natural_next is outside the called section's subtree, return instead
        if self.is_outside_section(natural_next, frame.called_section_id) {
            let return_id = frame.return_block_id;
            self.call_stack.pop();
            return Some(return_id);
        }
    }

    Some(natural_next)
}
```

#### 5. Section Boundary Detection

```rust
fn is_outside_section(&self, block_id: BlockId, section_id: BlockId) -> bool {
    // Walk up block_id's parent chain
    // If we encounter section_id, we're inside the section
    // If we hit root without finding it, we're outside

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
```

**Key insight:** A called section "completes" when natural traversal would move to a block outside that section's subtree. This works because:
- If section has children: we traverse them normally
- If section jumps with `->`: we execute that section, then continue
- When that jumped-to section finishes, next block is outside original called section
- We intercept this and return instead

#### 6. Modified skip() Logic

```rust
pub fn skip(&mut self) -> bool {
    let initial_stack_depth = self.call_stack.len();
    let previous_program_counter = self.program_counter;

    // Keep stepping until we reach END or return from current call
    while !self.has_ended() && self.can_continue() {
        self.step();

        // If in a call, stop when we return from it
        if self.call_stack.len() < initial_stack_depth {
            break;
        }
    }

    if self.program_counter > previous_program_counter {
        self.previous_program_counter = previous_program_counter;
    }

    true
}
```

The key change: skip stops when call stack depth decreases (we've returned from the called section). This prevents infinite loops when skipping through recursive calls like `<-> .`.

#### 7. Validation and Error Handling

Reuse existing validation from `GoToSection`:
- Section not found errors (compile-time)
- Path validation (spacing, empty references, etc.)
- Navigation above root errors

No new validation rules needed - `<->` has identical syntax requirements to `->`.

### Implementation Trade-offs

**Option A: Call Stack with Section Boundary Detection** (CHOSEN)
- ✅ Clean separation: parser creates blocks, runtime handles execution
- ✅ No modification to block tree structure
- ✅ Handles all edge cases (jumps, nesting, subsections)
- ✅ Reuses existing traversal logic
- ✅ Skip behavior naturally prevents infinite loops
- ✅ Aligns with existing architecture patterns
- ❌ Slightly more complex return logic (acceptable)

**Option B: Inject Return Marker Blocks**
- ❌ Modifies block tree during parsing
- ❌ Makes debugging harder (phantom blocks in tree)
- ❌ Complicates block ID references
- ✅ Simpler return logic (just hit marker block)

**Option C: Track Parent Section Instead of Called Section**
- ❌ Ambiguous when parent has multiple children
- ❌ Doesn't handle jumps correctly
- ❌ Breaks with complex control flow

## Decision

Implement **Option A**: Runtime call stack with section boundary detection.

This approach:
- Maintains clean separation between parsing and runtime
- Reuses existing path resolution and validation logic
- Handles all edge cases naturally
- Provides intuitive skip behavior with infinite loop protection
- Aligns with the project's architecture philosophy

### Implementation Steps

1. Add `GoToSectionAndBack` variant to `BlockType` enum
2. Add call stack fields to `Runtime` struct
3. Create `go_to_section_and_back_parser.rs` (reuse logic from existing parser)
4. Integrate parser into main parse loop
5. Reuse existing compile-time validation (no changes needed)
6. Implement call stack push/pop in `find_next_block()`
7. Implement section boundary detection helper
8. Modify `skip()` to respect call stack depth
9. Add unit tests for call stack and boundary detection
10. Create 28 compatibility tests for go-to-section-and-back feature

## Consequences

### Positive Outcomes

**1. Enables Narrative Composition**
- Reusable narrative fragments
- Function-like behavior in narratives
- Cleaner story structure through composition

**2. Maintains Architectural Consistency**
- Compile-time path resolution (same as `->`)
- No block tree modification
- Clean separation of parsing and runtime concerns

**3. Intuitive Behavior**
- Code after `<->` executes (unlike `->`)
- Nested calls work like function calls in programming
- Skip behavior prevents infinite loops naturally

**4. Reuses Existing Code**
- Path resolution logic
- Validation and error messages
- Compile-time checking

### Performance Considerations

**Call Stack Overhead:**
- Each `<->` call pushes one small frame (~16 bytes)
- Stack depth typically shallow (< 10 levels)
- Pop operation is O(1)
- Negligible performance impact

**Section Boundary Detection:**
- O(depth) operation where depth is block nesting level
- Called only when returning from calls
- Typically shallow trees (< 10 levels)
- Could cache section membership if needed (unlikely)

**Overall:** Performance impact is negligible for typical narrative scripts.

### Interaction with Existing Features

**Go To Section (`->`):**
- Works correctly within called sections
- Jump gets resolved, then return happens
- No conflicts or ambiguities

**Sections and Subsections:**
- Subsections execute normally within called sections
- Return happens after all subsections complete

**Comments:**
- No interaction - comments are ignored as usual

**Skip Command:**
- Behavior changes to respect call boundaries
- Prevents infinite loops with recursive calls
- More intuitive: "skip this section" not "skip entire script"

### Edge Cases Handled

**1. Recursive Calls (`<-> .`):**
Allowed and supported. Skip prevents infinite loops.

**2. Mutual Recursion:**
Supported through call stack. Skip prevents issues.

**3. END During Call:**
Script terminates immediately without returning (intuitive behavior).

**4. Empty Called Section:**
Immediately returns (existing empty section validation applies).

**5. Called Section Only Contains Calls:**
Nested calls execute correctly through call stack.

### Limitations and Future Considerations

**No Tail Call Optimization:**
Deep recursion could theoretically overflow call stack, but:
- Rust's heap-allocated Vec grows dynamically
- Typical narrative scripts won't recurse deeply
- Could add stack depth limit if needed

**No Call Context/Parameters:**
This ADR doesn't add parameters or context passing. Future ADRs could add:
- Variables passed to called sections
- Return values from called sections
- Local scope for called sections

**No Automatic Loop Detection:**
Infinite loops are possible but protected by skip behavior. Future could add:
- Maximum call depth limit
- Cycle detection warnings

### Testing Strategy

**Unit Tests (Runtime):**
- Call stack push/pop operations
- Section boundary detection algorithm
- Skip behavior with various call depths
- Edge cases (empty sections, END during call, etc.)

**Compatibility Tests:**
- Basic call and return scenarios
- Nested calls (2-3 levels)
- Integration with `->` jumps
- Skip behavior in various contexts
- Path resolution (absolute, relative, `..`, `.`)
- Error cases (same as `->`)

28 compatibility tests provide comprehensive coverage of all scenarios.

## Other Related ADRs

- [Go To Section Navigation](000013-go-to-section.md) - Foundation for path resolution and jumping
- [Sections and Navigation Support](000011-sections-and-navigation.md) - Section structure
- [Modular Parser Architecture](000010-modular-parser-architecture.md) - FeatureParser pattern

## References

- [Function calls and call stacks](https://en.wikipedia.org/wiki/Call_stack) - Traditional programming concept adapted for narratives
- [Ink tunnels](https://github.com/inkle/ink/blob/master/Documentation/WritingWithInk.md#tunnels) - Similar "call and return" concept in another narrative language
- [Subroutines in interactive fiction](https://inform7.com/book/WI_11_7.html) - Reusable narrative fragments
