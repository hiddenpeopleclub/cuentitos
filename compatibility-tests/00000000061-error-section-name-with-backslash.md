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
00000000061-error-section-name-with-backslash.cuentitos:1: ERROR: Section names cannot contain '\' character: Section \ Name
```
