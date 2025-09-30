# Section Nesting Error

This test verifies that the engine throws an error when section content
is not properly indented (nested) under its section header.

## Script
```cuentitos
# section_1: First Section
Some content without proper indentation

## subsection_1: First Subsection
  This line is properly indented
This line is not properly indented
```

## Input
```input
s
```

## Result
```result
2: ERROR: Content must be indented under its section (# section_1: First Section)
```
