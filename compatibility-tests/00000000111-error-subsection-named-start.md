# Error: Subsection Named START

This test verifies that a subsection cannot be named "START" (reserved word).

## Script
```cuentitos
# Parent
  ## START
  Text
```

## Input
```input
s
```

## Result
```result
test.cuentitos:2: ERROR: Section name "START" is reserved: START
```
