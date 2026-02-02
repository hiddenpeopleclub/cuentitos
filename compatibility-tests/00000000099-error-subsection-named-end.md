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
00000000099-error-subsection-named-end.cuentitos:2: ERROR: Section name "END" is reserved: END
```
