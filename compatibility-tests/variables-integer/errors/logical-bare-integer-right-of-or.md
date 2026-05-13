# Logical Operator Error: OR With Bare Integer on the Right

Logical operators combine **comparisons** on both sides. A `req` whose
right operand of `or` is a plain integer term (no comparison operator)
is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
---

Line.
  req health > 0 or shield
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-right-of-or.cuentitos:7: ERROR: Logical operator 'or' expects a comparison as each operand, not an integer expression.
```
