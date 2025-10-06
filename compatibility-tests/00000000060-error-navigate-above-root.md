# Error: Cannot Navigate Above Root

This test verifies that navigating above root level results in a compile error.

## Script
```cuentitos
# Root
  ## Section A
  Text in A
  -> .. \ ..
```

## Input
```input
s
```

## Result
```result
test.cuentitos:4: ERROR: Cannot navigate above root level
```
