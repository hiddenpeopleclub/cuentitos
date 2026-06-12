# Set Error: Int Literal on the RHS of a String Set

A `set` on a string variable accepts only a string expression (a
double-quoted literal or another string variable). There is no implicit
int-to-string coercion, so an integer literal on the RHS is a parse-time
type-mismatch error. This parallels the bool *literal* rule (see
`variables-bool/errors/set-int-literal-rhs.md`).

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = 1
Hello
```

## Input
```input
s
```

## Result
```result
set-cross-type-int-literal-rhs.cuentitos:4: ERROR: Type mismatch: 'set' expression for string name must be a string expression, but '1' is int.
```
