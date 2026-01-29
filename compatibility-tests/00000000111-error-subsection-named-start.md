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
00000000111-error-subsection-named-start.cuentitos:2: ERROR: Section name "START" is reserved: START
```
