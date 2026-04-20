# Require Error: Undeclared Variable on the LHS

A `req` whose LHS references a variable that was never declared is a
parse-time error.

## Script
```cuentitos
--- variables
int health = 10
---

Line.
  req mana > 0
```

## Input
```input
s
```

## Result
```result
undeclared-variable-lhs.cuentitos:6: ERROR: Undeclared variable: mana
```
