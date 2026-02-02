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
00000000093-error-call-invalid-path.cuentitos:3: ERROR: Expected section names separated by ' \\ '
```
