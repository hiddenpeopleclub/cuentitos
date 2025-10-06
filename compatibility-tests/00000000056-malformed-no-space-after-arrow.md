# Error: Malformed No Space After Arrow

This test verifies that missing space after arrow results in a parse error.

## Script
```cuentitos
# Section A
->Section B

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
