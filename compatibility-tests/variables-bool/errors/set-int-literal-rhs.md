# Set Error: Int Literal on the RHS of a Bool Set

A `set` on a bool variable accepts only a bool expression (`true`, `false`, or
another bool variable). An integer literal on the RHS is a parse-time
type-mismatch error, parallel to the bool *default* rule (see
`errors/default-not-bool-literal.md`).

## Script
```cuentitos
--- variables
bool door_open = false
---
set door_open = 1
Hello
```

## Input
```input
s
```

## Result
```result
set-int-literal-rhs.cuentitos:4: ERROR: Type mismatch: 'set' expression for bool door_open must be a bool expression, but '1' is int.
```
