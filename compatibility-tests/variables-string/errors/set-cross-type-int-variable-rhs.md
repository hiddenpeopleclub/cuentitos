# Set Error: Int Variable on the RHS of a String Set

A `set` on a string variable accepts only a string expression (a
double-quoted literal or another string variable). There is no implicit
int-to-string coercion, so referencing an int variable on the RHS of a
string `set` is a parse-time type-mismatch error. This parallels the bool
cross-type rule (see `variables-bool/errors/set-cross-type-int-variable-rhs.md`).

## Script
```cuentitos
--- variables
int count = 3
string name = "Aria"
---
set name = count
Hello
```

## Input
```input
s
```

## Result
```result
set-cross-type-int-variable-rhs.cuentitos:5: ERROR: Type mismatch: 'set' expression for string name must be a string expression, but count is int.
```
