# Error: `freq` Referencing an Undeclared Variable

A `freq` line naming a variable that was never declared in `--- variables`
is a parse-time error, the same as an undeclared variable referenced from a
`req`: the variables block is fully known before the story runs, so there is
no need to wait for runtime to know `unknown_var` was never declared.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = happy
Something happens.
  (50%) Branch A.
  (50%) Branch B.
    freq unknown_var happy 10
```

## Input
```input
s
```

## Result
```result
freq-undeclared-variable.cuentitos:8: ERROR: Undefined variable: 'unknown_var'.
```
