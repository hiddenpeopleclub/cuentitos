# Options

### Submitters

- Claude Code with Fran Tufro

## Change Log

- [draft] 2025-10-08 - Initial draft for options feature

## Referenced Use Case(s)

- [Basic Two Options](../../compatibility-tests/00000000102-basic-two-options.md)
- [Nested Options](../../compatibility-tests/00000000108-nested-options.md)
- [Options With Continuation](../../compatibility-tests/00000000106-options-with-continuation.md)
- [Error: Options Without Parent](../../compatibility-tests/00000000117-error-options-without-parent.md)
- [Skip Stops At Options](../../compatibility-tests/00000000114-skip-stops-at-options.md)

## Context

Interactive narratives often require player choices that branch the story. The cuentitos language currently lacks a mechanism for presenting choices to the player and handling their selections. This ADR proposes adding an Options feature that allows authors to:
- Present multiple choices to the player
- Branch execution based on player selection
- Nest options within other options for complex decision trees
- Mix options with narrative flow

Options are fundamental to interactive storytelling and enable non-linear narratives where player agency drives the story.

## Proposed Design

### Option Syntax

Options use asterisk syntax with one level of indentation beyond their parent:

```cuentitos
What do you want to do?
  * Go left
    You went left
  * Go right
    You went right
```

### Key Requirements

1. **Syntax**: Lines starting with `*` (after whitespace) followed by option text
2. **Indentation**: Options must be exactly one level (2 spaces) deeper than parent text
3. **Parent requirement**: Options MUST have a parent text - options at root level are invalid
4. **Sibling rules**: Options cannot have non-option siblings before them (but can have after)
5. **Children**: Options can have any valid blocks as children (text, sections, GoTos, nested options)
6. **Flow**: After option content executes, flow continues at parent level

### Runtime Behavior

**Display Format:**
```
Parent text
  1. Option one
  2. Option two
>
```

**Selection:**
- User enters option number (1, 2, 3, etc.)
- Selected option displays as: `Selected: [Option Text]`
- Option's child content then executes
- After children complete, execution continues at parent's next sibling

**Skip Behavior:**
- Skip mode (`s`) stops when encountering options
- User must manually select an option
- Skip can resume after selection

**Input Validation:**
- Invalid option number: `Invalid option: {input}` + re-display options
- Commands `n` or `s` at prompt: `Use option numbers (1-N) to choose (plus q to quit)`
- `q` command works at option prompt to quit

### Implementation Architecture

This feature requires changes across three layers:

#### 1. Common Layer - New BlockType

Add `Option` variant to `BlockType` enum:

```rust
pub enum BlockType {
    Start,
    String(StringId),
    Section { id: String, display_name: String },
    Option(StringId),  // NEW - text stored in strings table
    GoToSection { path: String, target_block_id: BlockId },
    GoToSectionAndBack { path: String, target_block_id: BlockId },
    End,
}
```

**Rationale:** Options are first-class blocks in the narrative structure with special semantics (choice points) that warrant their own block type. The option text is stored in the strings table for consistency with other text content.

#### 2. Parser Layer - OptionParser

Create `OptionParser` following the `FeatureParser` trait pattern:

**Location:** `parser/src/parsers/option_parser.rs`

**Responsibilities:**
- Detect lines starting with `*`
- Extract option text
- Return `OptionParseResult` with the option text

**Validation:**
Add validation logic in main parser (`parser/src/parser.rs`):
- Check if option has a parent (not at root level or after non-option sibling)
- Track whether we've seen a non-option child at current level
- Return `ParseError::OptionsWithoutParent` if invalid

**ParseError Addition:**
```rust
pub enum ParseError {
    // ... existing errors
    OptionsWithoutParent {
        file: Option<PathBuf>,
        line: usize,
    },
}
```

#### 3. Runtime Layer - Option Execution

**State Management:**
Add to `Runtime`:
```rust
pub struct Runtime {
    // ... existing fields
    waiting_for_option_selection: bool,
    current_options: Vec<BlockId>,  // IDs of available options
}
```

**Execution Flow:**
- When runtime encounters first option child, collect all option siblings
- Set `waiting_for_option_selection = true`
- Store option block IDs in `current_options`
- Return control to CLI (stop stepping)

**New Runtime Methods:**
```rust
pub fn is_waiting_for_option(&self) -> bool
pub fn get_current_options(&self) -> Vec<(usize, StringId)>  // (number, string_id)
pub fn select_option(&mut self, choice: usize) -> Result<(), String>
```

#### 4. CLI Layer - Option Interaction

**Display Logic:**
- Detect when runtime is waiting for option selection
- Display parent text, then numbered options with 2-space indent
- Display `> ` prompt
- Wait for user input

**Input Processing:**
- Parse option number from input
- Validate range (1 to N)
- Call `runtime.select_option(choice)`
- Display `Selected: [Option Text]`
- Continue execution

**Error Handling:**
- Invalid number: print error, re-display options
- `n` or `s`: print help message, re-display options
- `q`: quit as usual

### Implementation Options Considered

#### Option A: Store as String(StringId) with metadata (Rejected)

Use existing `String(StringId)` block type and track which strings are options via metadata.

**Pros:**
- Reuses existing block type
- Minimal changes to BlockType

**Cons:**
- Options are semantically different from narrative text
- Runtime needs external metadata to distinguish options from strings
- Complicates rendering logic
- Less clear data model
- Cannot extend with option-specific features easily

#### Option B: Option(StringId) as Dedicated BlockType (Recommended)

Add `Option(StringId)` to BlockType enum, storing text in strings table.

**Pros:**
- Clear semantic distinction between narrative and choices
- Self-documenting data structure
- Runtime can handle options explicitly via pattern matching
- Easier to extend with features (e.g., conditional options, disabled options)
- Matches the mental model: options ARE different from text
- Consistent with existing string storage strategy
- String deduplication benefits

**Cons:**
- Slightly more complex BlockType enum

**Decision:** Option B provides better semantics, extensibility, and consistency with existing architecture.

#### Option C: Options as Container with Child OptionItems (Rejected)

Create `OptionsGroup` block containing `OptionItem` children.

**Pros:**
- More structured representation
- Could support option metadata

**Cons:**
- Over-engineered for current needs
- Complicates parsing (need to track groups)
- Doesn't match the syntax (no group concept in script)
- Violates YAGNI

## Considerations

### Parent Validation Strategy

Options must have a parent text. This can fail in two scenarios:

**Scenario 1: Options at root level**
```cuentitos
* Option A
  Content
```

**Scenario 2: Non-option sibling before options**
```cuentitos
Parent text
  Regular text
  * Option A
    Content
```

In scenario 2, "Regular text" becomes the last block at that level, so `* Option A` has no valid parent (its parent would be "Parent text" but "Regular text" is between them).

**Implementation:** Track whether we've added a non-option child at the current level. If we have, and we encounter an option, emit `OptionsWithoutParent` error.

### Skip Behavior

Skip mode must stop at options because:
- Options require user input (non-deterministic)
- Skip is for fast-forwarding through deterministic content
- Stopping at choices is expected UX (user wants to make decisions)

### Why Not Use Numbers for Syntax?

Considered using `1.` , `2.` for options, but:
- Numbers imply ordering that author must maintain
- Inserting options breaks numbering
- `*` is common in markdown and familiar
- Parser auto-numbers at runtime

### Option Text Storage

Option text is stored in the strings table (as `StringId`) for consistency:
- Aligns with existing architecture (strings are centralized)
- Enables string deduplication if needed
- Simple runtime lookup via `database.strings[string_id]`
- Consistent with how other text content is stored

### Flow After Option

After an option's content executes, flow continues at the parent level. This enables:

```cuentitos
Question
  * Choice One
    Content one
  * Choice Two
    Content two
After all choices
```

Both choices lead to "After all choices" - common pattern in interactive fiction.

## Decision

**Implement Option B: Option(StringId) as Dedicated BlockType**

Add `Option(StringId)` to BlockType and implement:
1. `OptionParser` following FeatureParser pattern
2. Parent validation in main parser
3. Runtime state management for option selection
4. CLI option display and input handling

This approach provides clear semantics, good extensibility, and aligns with the mental model of options as distinct choice points in the narrative.

### Implementation Steps

1. **Common layer**: Add `Option(StringId)` to `BlockType`
2. **Parser layer**:
   - Create `OptionParser` in `parser/src/parsers/option_parser.rs`
   - Add `OptionsWithoutParent` to `ParseError`
   - Add validation logic in main parser
   - Register OptionParser in parse loop
   - Store option text in strings table and create Option block with StringId
3. **Runtime layer**:
   - Add option selection state to Runtime
   - Implement `is_waiting_for_option()`, `get_current_options()`, `select_option()`
   - Modify step/skip logic to stop at options
4. **CLI layer**:
   - Detect option waiting state
   - Display numbered options (lookup text via string_id)
   - Handle option input
   - Display selected option

## Other Related ADRs

- [Modular Parser Architecture](000010-modular-parser-architecture.md) - Pattern for OptionParser
- [Indentation Block Parenting](000009-indentation-block-parenting.md) - Parent-child relationships
- [Skip Feature](000007-skip-feature.md) - How skip interacts with options

## References

- [Ink](https://github.com/inkle/ink) - Similar choice syntax with `*` and `+`
- [Twine](https://twinery.org/) - Passage-based interactive fiction with links as choices
- [ChoiceScript](https://www.choiceofgames.com/make-your-own-games/choicescript-intro/) - Choice-driven narrative engine
