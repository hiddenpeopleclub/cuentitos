# Error: Section Not Found - Relative Path

This test verifies that referencing a non-existent section using a relative path results in a compile error.

## Script
```cuentitos
# Root
  ## Section A
  Text in A
  -> NonExistent
  ## Section B
  Text in B
```

## Input
```input
s
```

## Result
```result
test.cuentitos:4: ERROR: Section not found: NonExistent
```
