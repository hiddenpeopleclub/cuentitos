# Error: Empty Section

This test verifies that empty sections result in a compile error.

## Script
```cuentitos
# Section A

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
00000000062-error-empty-section.cuentitos:1: ERROR: Section must contain at least one block: Section A
```
