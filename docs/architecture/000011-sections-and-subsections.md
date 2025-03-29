# Sections and Sub-sections

This ADR describes the implementation of sections and sub-sections in the cuentitos language.

### Submitters

- Fran Tufro

## Change Log

- 2024-03-21 - [First draft of ADR created](https://github.com/hiddenpeopleclub/cuentitos/pull/XX)

## Use Case(s)

To organize content in a cuentitos script, we need a way to group related blocks of text and choices into logical sections. This helps with:

1. Story organization and readability
2. Navigation between different parts of the story
3. Creating hierarchical story structures
4. Future features like "go to section" and "return to section"

## Context

Currently, cuentitos scripts are linear sequences of blocks without any hierarchical organization. While indentation is used to establish parent-child relationships between blocks, there's no way to group these blocks into larger logical units.

Adding section support will allow writers to better organize their stories and enable future features that depend on section-based navigation.

## Proposed Design

### Section Syntax

Sections are defined using a markdown-like header syntax:

```cuentitos
# Section Title
Content in the section

## Sub-section Title
Content in the sub-section

### Deep Sub-section Title
Content in the deep sub-section
```

Key aspects of the design:

1. **Section Headers**
   - Start with one or more `#` characters
   - Must have a non-empty title
   - Number of `#` characters determines the section level
   - Must follow proper indentation rules (multiples of 2 spaces)
   - Section titles are stored in the strings table for localization
   - Section names must be unique within their level

2. **Section Hierarchy**
   - Sections can be nested to create sub-sections
   - Sub-sections must have a parent section
   - Each additional level is denoted by an extra `#` character
   - Indentation must match the section level

3. **Block Parenting**
   - All blocks belong to their nearest parent section
   - Blocks inherit the indentation level of their section
   - Blocks must maintain proper indentation relative to their section

4. **Error Handling**
   - Empty section titles are not allowed
   - Sub-sections cannot exist without a parent section
   - Section indentation must follow the 2-space rule
   - Section levels must increase gradually (no skipping levels)
   - Duplicate section names at the same level are not allowed
   - All errors must include line numbers and reference to original definitions
   - Multiple errors in the same file must all be reported

### Implementation Plan

The implementation will be divided into phases to manage complexity and ensure proper testing at each step:

#### Phase 1: Parser Extensions
1. Add `Section` block type to represent sections in the AST
2. Extend the parser to recognize section syntax (`#`, `##`, etc.)
3. Implement section title extraction and validation
4. Add section titles to the strings table
5. Implement indentation validation for sections
6. Add section hierarchy validation
7. Add section name uniqueness validation
8. Run basic section parsing tests

#### Phase 2: Block Hierarchy
1. Extend block parenting logic to handle sections
2. Implement section level tracking
3. Add validation for block indentation within sections
4. Add validation for section nesting rules
5. Run nested section tests

#### Phase 3: Runtime Support
1. Update runtime to handle section blocks
2. Implement section title display in output
3. Add section state tracking in runtime
4. Ensure proper section traversal
5. Run full compatibility test suite

#### Phase 4: Error Handling
1. Implement specific error types for section-related issues:
   - Empty section titles
   - Invalid section hierarchy
   - Incorrect indentation
   - Duplicate section names
2. Add detailed error messages with:
   - Current line number
   - Reference to original definition ("Previously defined at line X")
   - Parent section context when relevant
3. Implement multiple error collection and reporting
4. Run error case tests

#### Phase 5: Integration & Testing
1. Update existing tests to handle sections
2. Verify i18n integration with section titles
3. Test section behavior with existing features
4. Run full regression test suite
5. Update documentation with section examples

Each phase will follow the TDD approach:
1. Run tests (they should fail)
2. Implement the feature
3. Run tests again (they should pass)
4. Refactor if needed
5. Document any necessary changes or decisions

### Compatibility Tests

The feature is defined by five compatibility tests:

1. Basic section support (`00000000011-basic-section.md`)
   - Tests simple sections with content
   - Verifies section titles in output
   - Checks multiple sections in a file

2. Nested sections (`00000000012-nested-sections.md`)
   - Tests multiple levels of section nesting
   - Verifies proper section hierarchy
   - Checks sibling sections at the same level

3. Section without title error (`00000000013-section-without-title.md`)
   - Verifies error handling for empty section titles
   - Ensures proper error message format

4. Sub-section without parent error (`00000000014-subsection-without-parent.md`)
   - Tests error handling for orphaned sub-sections
   - Verifies proper section hierarchy enforcement

5. Invalid section indentation error (`00000000015-invalid-section-indentation.md`)
   - Tests error handling for incorrect indentation
   - Ensures consistency with existing indentation rules

6. Duplicate section names - same parent (`00000000016-duplicate-section-names.md`)
   - Tests error handling for duplicate section names at the same level
   - Verifies proper error message format with original definition line
   - Ensures uniqueness is enforced per level under same parent

7. Duplicate section names - root level (`00000000017-duplicate-section-names-root.md`)
   - Tests error handling for duplicate section names at root level
   - Verifies proper error message format for root-level duplicates
   - Ensures root level section name uniqueness

8. Different levels allowed (`00000000018-duplicate-section-names-different-levels.md`)
   - Verifies that same names are allowed at different levels
   - Tests same names under different parents
   - Ensures proper nesting with repeated names

9. Multiple duplicate errors (`00000000019-duplicate-section-names-multiple-errors.md`)
   - Tests reporting of multiple duplicate section errors
   - Verifies all errors are reported with proper line numbers
   - Ensures proper error context for each duplicate

## Considerations

1. **Parser Integration**
   - Section parsing must integrate with existing block parsing
   - Must maintain compatibility with current indentation rules
   - Should reuse existing error handling patterns
   - Section titles must be added to the strings table like any other text

2. **Runtime Behavior**
   - Sections should be treated as organizational units
   - Section titles should be displayed in output
   - Future features may use sections for navigation
   - Section titles should respect the current language setting

3. **Error Messages**
   - Should follow existing error message format
   - Must provide clear guidance for fixing issues
   - Should maintain line number accuracy

4. **Future Extensions**
   - Design should support future "go to section" feature
   - Should allow for section-based state management
   - May enable section-based story branching

## Decision

We will implement sections as described in this ADR, following the test-driven development approach established by the compatibility tests. The implementation will focus on:

1. Extending the parser to handle section syntax
2. Implementing proper error handling
3. Maintaining section hierarchy in the runtime
4. Displaying section titles in output

## Other Related ADRs

- [Compatibility Tests](./000001-compatibility-tests.md) - Defines the testing framework used
- [Parser](./000004-parser.md) - Describes the parser that will be extended
- [Indentation Block Parenting](./000009-indentation-block-parenting.md) - Describes indentation rules that sections must follow
- [I18n Strings](./000008-i18n-strings.md) - Describes how strings (including section titles) are localized
