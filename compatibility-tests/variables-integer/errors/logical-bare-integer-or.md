# Logical Operator Error: OR Applied to a Bare Integer Expression

Logical operators combine **comparisons**, not bare integer expressions. A
`req` whose left operand of `or` is a plain integer term (no comparison
operator) is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
---

Line.
  req health or shield > 0
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-or.cuentitos:7: ERROR: Logical operator 'or' expects a comparison as each operand, not an integer expression.
```
