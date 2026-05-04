# Logical Operator Error: OR Applied to a Bare Integer Expression

Logical operators combine **comparisons**, not bare integer expressions. A
`req` whose left operand of `OR` is a plain integer term (no comparison
operator) is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
---

Line.
  req health OR shield > 0
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-or.cuentitos:7: ERROR: Logical operator 'OR' expects a comparison on its left, not an integer expression.
```
