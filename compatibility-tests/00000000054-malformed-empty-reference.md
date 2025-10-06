# Error: Malformed Empty Reference

This test verifies that an empty section reference results in a parse error.

## Script
```cuentitos
# Section A
->

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
