# Require Error: Undeclared Variable Inside an Arithmetic Expression

A `req` whose RHS expression mixes literals and an undeclared variable is a
parse-time error.

## Script
```cuentitos
--- variables
int health = 10
---

Line.
  req health > 5 + mana
```

## Input
```input
s
```

## Result
```result
undeclared-variable-in-expression.cuentitos:6: ERROR: Undeclared variable: mana
```
