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
00000000098-error-section-named-end.cuentitos:1: ERROR: Section name "END" is reserved: END
```
