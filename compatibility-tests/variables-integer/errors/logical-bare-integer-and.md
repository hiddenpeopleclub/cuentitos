# Logical Operator Error: AND Applied to a Bare Integer Expression

Logical operators combine **comparisons**, not bare integer expressions. A
`req` whose left operand of `AND` is a plain integer term (no comparison
operator) is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
---

Line.
  req health AND shield > 0
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-and.cuentitos:7: ERROR: Logical operator 'AND' expects a comparison on its left, not an integer expression.
```
