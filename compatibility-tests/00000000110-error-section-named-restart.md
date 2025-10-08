# Error: Section Named RESTART

This test verifies that a section cannot be named "RESTART" (reserved word).

## Script
```cuentitos
# RESTART
Text
```

## Input
```input
s
```

## Result
```result
test.cuentitos:1: ERROR: Section name "RESTART" is reserved: RESTART
```
