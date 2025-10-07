# Error: Subsection Named END

This test verifies that a subsection cannot be named "END" (reserved word).

## Script
```cuentitos
# Parent
  ## END
  Text
```

## Input
```input
s
```

## Result
```result
test.cuentitos:2: ERROR: Section name "END" is reserved: END
```
