# Require Error: Undeclared Variable on the RHS

A `req` whose RHS references a variable that was never declared is a
parse-time error.

## Script
```cuentitos
--- variables
int health = 10
---

Line.
  req health > mana
```

## Input
```input
s
```

## Result
```result
undeclared-variable-rhs.cuentitos:6: ERROR: Undefined variable: 'mana'.
```
