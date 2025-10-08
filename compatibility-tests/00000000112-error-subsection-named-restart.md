# Error: Subsection Named RESTART

This test verifies that a subsection cannot be named "RESTART" (reserved word).

## Script
```cuentitos
# Parent
  ## RESTART
  Text
```

## Input
```input
s
```

## Result
```result
test.cuentitos:2: ERROR: Section name "RESTART" is reserved: RESTART
```
