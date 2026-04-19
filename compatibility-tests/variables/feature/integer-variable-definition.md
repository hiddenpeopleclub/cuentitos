# Integer variable definition

This test checks the definition of integer variables in the script with their default value.

## Pending

Variables are not implemented yet in the v0.3 runtime — the runtime echoes the
`--- variables` block as literal text. Re-enable this test once variable
declarations are parsed and printed at `?` input.

## Script
```cuentitos
--- variables
int an_integer
int an_integer_that_starts_with_one = 1
---
```

## Input
```input
?
```

## Result
```result
an_integer: 0
an_integer_that_starts_with_one: 1
```
