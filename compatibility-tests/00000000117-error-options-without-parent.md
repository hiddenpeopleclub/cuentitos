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
Error: Options must have a parent
```
