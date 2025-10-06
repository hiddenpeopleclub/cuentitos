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
test.cuentitos:1: ERROR: Section must contain at least one block: Section A
```
