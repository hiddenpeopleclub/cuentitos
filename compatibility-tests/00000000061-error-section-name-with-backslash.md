# Error: Section Name Contains Backslash

This test verifies that section names containing backslash result in a parse error.

## Script
```cuentitos
# Section \ Name
Text in section
```

## Input
```input
s
```

## Result
```result
test.cuentitos:1: ERROR: Section names cannot contain '\' character: Section \ Name
```
