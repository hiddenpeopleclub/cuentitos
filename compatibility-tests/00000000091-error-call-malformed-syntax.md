# Error: Call Malformed Syntax

This test verifies that malformed call syntax (no section name) produces an error.

## Script
```cuentitos
# Section A
In A
<->
```

## Input
```input
s
```

## Result
```result
test.cuentitos:3: ERROR: Expected section name after '<->'
```
