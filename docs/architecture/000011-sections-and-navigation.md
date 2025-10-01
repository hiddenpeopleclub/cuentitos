# Sections and Navigation Support

### Submitters

- Fran Tufro

## Change Log

- [approved] 2025-03-06 - Initial implementation using FeatureParser architecture

## Referenced Use Case(s)

- [Basic Section Test](../../compatibility-tests/00000000011-basic-section.md)
- [Nested Sections Test](../../compatibility-tests/00000000012-nested-sections.md)
- [Section Without Title Error](../../compatibility-tests/00000000013-section-without-title.md)
- [Subsection Without Parent Error](../../compatibility-tests/00000000014-subsection-without-parent.md)

## Context

Interactive narratives often require organizational structure beyond simple linear progression. Sections provide a way to organize content into logical groups, create navigable story segments, and establish clear hierarchies within the narrative. This enables features like chapter-based navigation, menu systems, and branching storylines.

## Proposed Design

### Section Syntax

Sections use markdown-style header syntax (with optional custom IDs):
```cuentitos
# Display Name
  Content within the section

# section_id: Display Name
  Content within the section (with explicit ID)

## Subsection Display Name
  Content within subsection

### Deep Subsection
  Content at third level
```

### Core Implementation

1. **Section Block Type**
   - Added new `BlockType::Section { id: String, display_name: String }`
   - Sections have both an identifier and display name
   - When no explicit ID is provided, the display name is used as the ID

2. **Parser Architecture Using FeatureParser**
   - Implemented `SectionParser` using the modular `FeatureParser` trait
   - Located in `parser/src/parsers/section_parser.rs`
   - Returns `Option<SectionParseResult>` with id, display_name, and hash_count
   - Integrates cleanly with the existing modular parser architecture

3. **Hierarchy Management**
   - Dual-stack approach:
     - `last_block_at_level`: Tracks regular content hierarchy
     - `last_section_at_level`: Tracks section hierarchy separately
   - Sections find parents based on indentation level
   - Content blocks find parents based on indentation

4. **Section Level Determination**
   - Section level is determined by indentation (not hash count)
   - Hash count is used only for validation (detecting orphaned subsections)
   - This aligns with the indentation-based hierarchy system

5. **Validation Rules**
   - **Empty titles**: Sections without titles produce `SectionWithoutTitle` error
   - **Orphaned subsections**: Subsections (## or more) without a parent section produce `InvalidSectionHierarchy` error
   - **Duplicate names**: Sections with the same display name at the same level under the same parent produce `DuplicateSectionName` error
   - Different hierarchy levels can have the same section names (allowed)

6. **Error Collection**
   - Parser collects multiple errors instead of failing on the first error
   - Multiple errors are reported together in `MultipleErrors` variant
   - Allows developers to see all issues at once

### Runtime Rendering

Sections are rendered with hierarchical paths using backslash separators:
```
-> First Section
-> First Section \ Subsection
-> First Section \ Subsection \ Deep Subsection
```

### Key Design Decisions

1. **FeatureParser Integration**: Using the trait-based architecture allows clean separation and extensibility
2. **Error Collection**: Collecting errors provides better developer experience
3. **Indentation-based levels**: Maintains consistency with the rest of the language
4. **Hash count for validation only**: Provides semantic meaning while keeping level determination simple
5. **Duplicate name detection**: Prevents confusing navigation scenarios

## Considerations

- **Navigation**: This ADR only covers section definition. Future ADRs will cover "Go To Section" navigation
- **Options**: Sections will integrate with future option/choice systems
- **Variables**: Section scope for variables will be addressed in variable ADRs

## Decision

Implemented sections using the modular `FeatureParser` architecture with dual-stack hierarchy management. This maintains clean separation between section hierarchy and content hierarchy while enabling robust validation and error reporting.

## Other Related ADRs

- [Modular Parser Architecture](000010-modular-parser-architecture.md) - Foundation for the FeatureParser trait
- [Indentation and Block Parenting](000009-indentation-block-parenting.md) - Foundation for hierarchical structures
- [Parser](000004-parser.md) - Original parser architecture
- [Lines of Text](000005-lines-of-text.md) - How text content interacts with sections

## References

- [Markdown Headers](https://www.markdownguide.org/basic-syntax/#headings) - Inspiration for section syntax
