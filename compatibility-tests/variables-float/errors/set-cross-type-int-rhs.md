# Set Error: Int Variable on the RHS of a Float Set

A `set` on a float variable accepts only a float expression. There is no
implicit int-to-float coercion (the same rule the float *defaults* enforce,
see `errors/cross-type-default.md`), so referencing an int variable on the
RHS of a float `set` is a parse-time type-mismatch error.

## Script
```cuentitos
--- variables
int count = 3
float ratio = 0.0
---
set ratio = count
Hello
```

## Input
```input
s
```

## Result
```result
set-cross-type-int-rhs.cuentitos:5: ERROR: Type mismatch: 'set' expression for float ratio must be a float expression, but count is int.
```
