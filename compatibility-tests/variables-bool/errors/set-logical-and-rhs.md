# Set Error: Logical `and` on the RHS of a Bool Set

The RHS of a bool `set` is `true`, `false`, or another bool variable — never an
inline logical expression. Logical operators (`and`, `or`, `not`) belong in
`req` conditions, so `and` on the RHS of a `set` is a parse-time error. This
parallels the bool *default* rule (see `errors/default-uses-logical-or.md`).

## Script
```cuentitos
--- variables
bool a = true
bool b = false
bool target = false
---
set target = a and b
Hello
```

## Input
```input
s
```

## Result
```result
set-logical-and-rhs.cuentitos:6: ERROR: Logical operators (and/or/not) are not allowed in 'set' expressions; use 'req' for boolean expressions.
```
