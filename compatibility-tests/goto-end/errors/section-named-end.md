# Error: Section Named END

This test verifies that a section cannot be named "END" (reserved word).

## Script
```cuentitos
# END
Text
```

## Input
```input
s
```

## Result
```result
section-named-end.cuentitos:1: ERROR: Section name "END" is reserved: END
```
