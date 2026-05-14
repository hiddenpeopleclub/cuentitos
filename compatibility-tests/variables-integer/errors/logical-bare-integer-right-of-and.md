# Logical Operator Error: AND With Bare Integer on the Right

Logical operators combine **comparisons** on both sides. A `req` whose
right operand of `and` is a plain integer term (no comparison operator)
is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
---

Line.
  req health > 0 and shield
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-right-of-and.cuentitos:7: ERROR: Logical operator 'and' expects a comparison as each operand, not an integer expression.
```
