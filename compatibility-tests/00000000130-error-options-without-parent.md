# Error: Options Without Parent

Options at root level should produce validation error.

## Script
```cuentitos
* Option A
  Content A
* Option B
  Content B
```

## Input
```input
s
```

## Result
```result
00000000130-error-options-without-parent.cuentitos:1: ERROR: Options must have a parent

00000000130-error-options-without-parent.cuentitos:3: ERROR: Options must have a parent
```
