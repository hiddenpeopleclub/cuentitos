# Comment Support

### Submitters

- Fran Tufro

## Change Log

- [approved] 2025-10-02 - Comment support feature implemented and tested

## Referenced Use Case(s)

- [Single Comment Line](../../compatibility-tests/00000000020-single-comment-line.md)
- [Comment Before Text](../../compatibility-tests/00000000021-comment-before-text.md)
- [Comment Between Text Lines](../../compatibility-tests/00000000023-comment-between-text-lines.md)
- [Comments in Sections](../../compatibility-tests/00000000025-comments-in-sections.md)
- [Comments with Arbitrary Indentation](../../compatibility-tests/00000000027-comments-with-arbitrary-indentation.md)

## Context

Cuentitos scripts currently lack a mechanism for adding explanatory notes, documentation, or temporarily disabling code without removing it. Comments are a fundamental feature in programming languages and authoring environments that enable developers to:
- Document the purpose and behavior of script sections
- Add notes for future reference
- Temporarily disable content during development
- Improve script readability and maintainability

This ADR proposes adding comment support using `//` syntax, where comments are completely ignored by the parser and can appear at any indentation level.

## Proposed Design

### Comment Syntax

Comments use double-slash syntax:
```cuentitos
// This is a comment
Text line
// Another comment

# Section
  // Comment within section
  Text in section
```

### Key Requirements

1. **Syntax**: Lines starting with `//` (after any whitespace) are comments
2. **Placement**: Comments must be on their own line (not inline after content)
3. **Indentation**: Comments can appear at any indentation level, not bound by hierarchy rules
4. **Multiple slashes**: `///`, `////`, etc. are all valid comments
5. **Empty comments**: `//` alone with no text is valid
6. **Parser behavior**: Comments are completely ignored and not stored in the Database

### Implementation Options

#### Option 1: Early Skip in Parse Loop (Recommended)

Check if a line is a comment immediately after reading it in the main parse loop, before indentation validation. If it's a comment, skip to the next line.

**Pros:**
- Simple and clean
- Comments truly ignored (similar to blank lines)
- No indentation validation applied (enables arbitrary indentation)
- Minimal code changes
- Treats comments as non-content (philosophically correct)

**Cons:**
- Doesn't follow the FeatureParser trait pattern

**Implementation Location:**
- Add helper method `is_comment(line: &str) -> bool`
- Add check in `Parser::parse()` main loop before indentation parsing

#### Option 2: Create CommentParser with FeatureParser Trait

Implement a `CommentParser` following the existing `FeatureParser` pattern like `SectionParser` and `LineParser`.

**Pros:**
- Consistent with existing parser architecture
- Extensible if comments need features later
- Follows established patterns

**Cons:**
- More complex for something that should be ignored
- Still requires special handling for indentation validation
- Comments would need to go through the full parsing pipeline just to be discarded
- Philosophical mismatch: FeatureParsers create content, comments are non-content

#### Option 3: Pre-process Script to Remove Comments

Strip comments from the input before passing to the parser.

**Pros:**
- Parser doesn't need to know about comments
- Very clean separation

**Cons:**
- Line numbers in error messages become incorrect
- More complex overall architecture
- Preprocessing step adds overhead

## Considerations

### Why `//` instead of `#`?

Initially considered using `#` for comments, but discovered that `#` is already used for section headers in the language. Using `//` avoids this conflict and aligns with common programming language conventions (C++, JavaScript, Rust, etc.).

### Indentation Validation

Since comments can have arbitrary indentation (not following hierarchy rules), they must be detected and skipped **before** indentation validation occurs. This is a key difference from other content types.

### Parser Architecture Consistency

While the codebase uses the `FeatureParser` trait pattern for extensible parsing, comments are fundamentally different - they are non-content that should be eliminated, not parsed and stored. An early skip is more appropriate than a full parser implementation.

### Future Extensibility

If comments ever need to support features like:
- Documentation generation
- Special comment directives
- Block comments (`/* */`)

The early-skip approach can be refactored to use a CommentParser at that time. YAGNI principle applies: don't add complexity for hypothetical future needs.

## Decision

**Implement Option 1: Early Skip in Parse Loop**

Comments will be detected and skipped early in the parse loop, before indentation validation. This approach:
- Achieves the goal of completely ignoring comments
- Allows arbitrary indentation without validation errors
- Keeps implementation simple and maintainable
- Treats comments philosophically correctly as non-content

### Implementation Details

1. Add helper method to `Parser`:
   ```rust
   fn is_comment(line: &str) -> bool {
       line.trim_start().starts_with("//")
   }
   ```

2. In `Parser::parse()` main loop, check for comments after reading each line:
   ```rust
   for line in script.as_ref().lines() {
       // Check if line is a comment and skip it
       if Self::is_comment(line) {
           context.current_line += 1;
           continue;
       }

       // ... rest of existing parsing logic
   }
   ```

3. No changes needed to:
   - Database structure
   - Block types
   - FeatureParser implementations
   - Runtime execution

## Other Related ADRs

- [Modular Parser Architecture](000010-modular-parser-architecture.md) - Context for parser design patterns
- [Sections and Navigation](000011-sections-and-navigation.md) - Why `#` was unavailable for comments
- [Parser](000004-parser.md) - Original parser architecture

## References

- [C++ Comments](https://en.cppreference.com/w/cpp/comment) - Similar `//` syntax
- [Rust Comments](https://doc.rust-lang.org/reference/comments.html) - Documentation comments with `///`
