# Error: Call Invalid Path Syntax

This test verifies that invalid path syntax produces an error.

## Script
```cuentitos
# Section A
In A
<-> \
```

## Input
```input
s
```

## Result
```result
test.cuentitos:3: ERROR: Expected section name after '<->'
```
