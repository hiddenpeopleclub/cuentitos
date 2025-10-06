# Error: Malformed Trailing Backslash

This test verifies that a trailing backslash in a section reference results in a parse error.

## Script
```cuentitos
# Section A
-> Section B \

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
test.cuentitos:2: ERROR: Expected section name after '->'
```
