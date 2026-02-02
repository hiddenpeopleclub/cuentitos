# Error: Call Navigate Above Root

This test verifies that trying to navigate above root with .. produces an error.

## Script
```cuentitos
# Section A
In A
<-> ..
```

## Input
```input
s
```

## Result
```result
00000000092-error-call-navigate-above-root.cuentitos:3: ERROR: Cannot navigate above root level
```
