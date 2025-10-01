# Cuentitos Language Documentation

This document describes the currently implemented features of the Cuentitos language based on the compatibility tests.

## Basic Structure

### Text Blocks

The most basic element in Cuentitos is a text block. Text blocks are simply lines of text that will be displayed in sequence:

```cuentitos
This is a single line of text
This is another line of text
```

### Indentation and Nesting

Text blocks can be nested using indentation. The indentation must be exactly 2 spaces per level. Using any other number of spaces (like 3) will result in a parse error.

Example of correct nesting:

```cuentitos
This is a parent string
  This is a child string
    This is a grandchild string
  This is another child string
```

Example that would cause an error:

```cuentitos
First line
   Invalid indentation  # Error: 3 spaces instead of 2
```

Nesting creates parent-child relationships between blocks, which affects navigation and flow control.

## Sections

Sections organize content into navigable segments using markdown-style headers with custom IDs:

```cuentitos
# section_1: First Section
  Content within the first section

## subsection_1: First Subsection
  Content within a subsection

### deep_section: Deeply Nested Section
  Content at the third level

## subsection_2: Second Subsection
  More content here

# section_2: Second Section
  Content in the second section
```

### Section Syntax Rules

1. **Format**: `# section_id: Display Name`
   - The `section_id` is used for programmatic reference (navigation, jumps)
   - The `Display Name` is shown to users

2. **Hierarchy Levels**:
   - `#` = Top-level section (level 0)
   - `##` = Subsection (level 1)
   - `###` = Sub-subsection (level 2)
   - And so on...

3. **Content Indentation**:
   - Content following a section header **must** be indented with at least 2 spaces
   - Sections can follow other sections without additional indentation

4. **Section Nesting**:
   - Sections automatically nest based on their header level
   - A `##` section becomes a child of the nearest preceding `#` section
   - A `###` section becomes a child of the nearest preceding `##` section

Example with error:
```cuentitos
# section_1: First Section
Some content without indentation  # ERROR: Content must be indented under its section
```

Correct version:
```cuentitos
# section_1: First Section
  Some content with proper indentation  # OK: Content is indented
```

### Runtime Behavior

When navigating through sections, the runtime displays:
- Section entry messages showing the hierarchy path
- Example: `Entered Section: First Section > Subsection (section_1/subsection_1)`

Sections will be essential for future navigation features like "Go To Section" and menu systems
