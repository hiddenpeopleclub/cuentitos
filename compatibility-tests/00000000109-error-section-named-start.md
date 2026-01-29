# Error: Section Named START

This test verifies that a section cannot be named "START" (reserved word).

## Script
```cuentitos
# START
Text
```

## Input
```input
s
```

## Result
```result
00000000109-error-section-named-start.cuentitos:1: ERROR: Section name "START" is reserved: START
```
