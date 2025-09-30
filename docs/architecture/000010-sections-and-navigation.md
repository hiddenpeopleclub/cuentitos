# Sections and Navigation Support

### Submitters

- Fran Tufro

## Change Log

- [approved] 2025-03-06 - Initial implementation completed

## Referenced Use Case(s)

- [Sections and Sub-sections Test](../../compatibility-tests/00000000011-sections-and-subsections.md)
- [Section Nesting Error Test](../../compatibility-tests/00000000012-section-nesting-error.md)

## Context

Interactive narratives often require organizational structure beyond simple linear progression. Sections provide a way to organize content into logical groups, create navigable story segments, and establish clear hierarchies within the narrative. This enables features like chapter-based navigation, menu systems, and branching storylines.

## Proposed Design

### Section Syntax

Sections use markdown-style header syntax with custom IDs:
```cuentitos
# section_id: Display Name
  Content within the section

## subsection_id: Subsection Display Name
  Content within subsection

### deep_section_id: Deeply Nested Section
  Content at third level
```

### Core Implementation

1. **Section Block Type**
   - Added new `BlockType::Section { id: String, display_name: String }`
   - Sections have both an identifier (for programmatic reference) and display name (for user display)

2. **Parser Architecture**
   - Implemented extensible block parser system in `parser/src/block_parsers/`
   - Each parser (SectionParser, StringParser) handles specific line types
   - Parser tries each block parser in order until one succeeds

3. **Hierarchy Management**
   - Dual-stack approach:
     - `last_block_at_level`: Tracks regular content hierarchy
     - `last_section_at_level`: Tracks section hierarchy separately
   - Sections find parents based on section depth (# count)
   - Content blocks find parents based on indentation

4. **Section Level Determination**
   - Section level = number of `#` symbols - 1
   - `#` = level 0, `##` = level 1, `###` = level 2
   - Indentation of the section line itself is ignored

5. **Content Validation**
   - Content following a section must be indented more than the section line
   - Sections can follow sections without additional indentation
   - Empty lines are ignored for validation

### Runtime Rendering

Sections are rendered with hierarchical paths:
```
Entered Section: First Section (section_1)
Entered Section: First Section > Subsection (section_1/subsection_1)
```

## Considerations

- **Navigation**: This ADR only covers section definition. Future ADRs will cover "Go To Section" navigation
- **Options**: Sections will integrate with future option/choice systems
- **Variables**: Section scope for variables will be addressed in variable ADRs

## Decision

Implemented the dual-stack approach to maintain clean separation between section hierarchy and content hierarchy. This allows sections to maintain their structural relationships while content can be freely organized within them.

## Other Related ADRs

- [Indentation and Block Parenting](000009-indentation-block-parenting.md) - Foundation for hierarchical structures
- [Parser](000004-parser.md) - Parser architecture that was extended for sections
- [Lines of Text](000005-lines-of-text.md) - How text content interacts with sections

## References

- [Markdown Headers](https://www.markdownguide.org/basic-syntax/#headings) - Inspiration for section syntax