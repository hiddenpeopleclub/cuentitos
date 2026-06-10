# Set Error: Logical `not` on the RHS of a Bool Set

The RHS of a bool `set` is `true`, `false`, or another bool variable — never an
inline logical expression. The unary `not` operator belongs in `req`
conditions, so `not` on the RHS of a `set` is a parse-time error. This
parallels the bool *default* rule (see `errors/default-uses-logical-not.md`).

## Script
```cuentitos
--- variables
bool a = true
bool target = false
---
set target = not a
Hello
```

## Input
```input
s
```

## Result
```result
set-logical-not-rhs.cuentitos:5: ERROR: Logical operators (and/or/not) are not allowed in 'set' expressions; use 'req' for boolean expressions.
```
