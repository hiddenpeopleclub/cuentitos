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

Nesting will be useful later when discussing flow (Sections, Choices, etc.)
