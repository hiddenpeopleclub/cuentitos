# Indentation and Block Parenting Support

### Submitters

- Fran Tufro

## Change Log

- [pending] 2024-03-05

## Context

The cuentitos game narrative engine needs a clear and consistent way to represent hierarchical relationships between story blocks through indentation, enabling proper block parenting and nested story structures.

## Proposed Design

### Core Changes

1. Indentation Rules
   - Use spaces for indentation (2 spaces per level)
   - Indentation level determines parent-child relationships
   - Each indentation level represents a deeper nesting in the story structure
   - Empty lines do not affect indentation context
   - Inconsistent indentation should raise compilation errors

2. Block Parenting
   - A block at indentation level N is a child of the nearest block at level N-1
   - Blocks at the same indentation level are siblings
   - Root-level blocks (no indentation) have no parent

3. Script Syntax Example
   ```cuentitos
   # room_description
   You enter a dimly lit room.
     * Look around
       The room has old furniture covered in dust.
       * Examine the desk
         You find a key in one of the drawers.
       * Check the bookshelf
         Most books are too damaged to read.
     * Leave immediately
       You decide it's better not to disturb this place.
   ```

4. Parser Changes
   - Track indentation level during parsing
   - Build parent-child relationships based on indentation
   - Validate indentation consistency
   - Generate clear error messages for indentation issues

5. Runtime Changes
   - Implement block traversal considering parent-child relationships
   - Maintain block hierarchy during story state management

### Configuration

No specific configuration needed for indentation rules, as they are part of the core syntax.

## Other Related ADRs

- [Lines of Text](000005-lines-of-text.md) - Text handling that will interact with indentation
- [I18n Strings](000008-i18n-strings.md) - String handling that must respect indentation

## References

- [Python Indentation Guide](https://www.python.org/dev/peps/pep-0008/#indentation) - Inspiration for indentation rules
