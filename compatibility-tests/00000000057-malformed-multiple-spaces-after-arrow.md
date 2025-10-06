# Error: Malformed Multiple Spaces After Arrow

This test verifies that multiple spaces after arrow results in a parse error.

## Script
```cuentitos
# Section A
->  Section B

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
