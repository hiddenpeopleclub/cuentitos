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
test.cuentitos:1: ERROR: Options must have a parent

test.cuentitos:3: ERROR: Options must have a parent
```
