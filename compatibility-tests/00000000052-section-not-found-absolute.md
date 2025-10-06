# Error: Section Not Found - Absolute Path

This test verifies that referencing a non-existent section using an absolute path results in a compile error.

## Script
```cuentitos
# Section A
-> NonExistent Section

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
test.cuentitos:2: ERROR: Section not found: NonExistent Section
```
