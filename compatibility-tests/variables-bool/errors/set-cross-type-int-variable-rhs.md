# Set Error: Int Variable on the RHS of a Bool Set

A `set` on a bool variable accepts only a bool expression (`true`, `false`, or
another bool variable). There is no implicit int-to-bool coercion, so
referencing an int variable on the RHS of a bool `set` is a parse-time
type-mismatch error. This parallels the int *literal* rule (see
`errors/set-int-literal-rhs.md`) and the float cross-type rule (see
`variables-float/errors/set-cross-type-int-rhs.md`).

## Script
```cuentitos
--- variables
int count = 3
bool door_open = false
---
set door_open = count
Hello
```

## Input
```input
s
```

## Result
```result
set-cross-type-int-variable-rhs.cuentitos:5: ERROR: Type mismatch: 'set' expression for bool door_open must be a bool expression, but count is int.
```
