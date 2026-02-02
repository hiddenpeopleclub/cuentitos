# Error: Cannot Use .. at Root Level

This test verifies that using .. at root level results in a compile error.

## Script
```cuentitos
# Section A
Text in A
-> ..

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
00000000059-error-parent-at-root.cuentitos:3: ERROR: Cannot navigate above root level
```
