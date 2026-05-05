# Logical Operator Error: NOT Applied to a Bare Integer Expression

Logical operators combine **comparisons**, not bare integer expressions. A
`req` whose `not` operand is a plain integer term (no comparison operator)
is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
---

Line.
  req not health
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-not.cuentitos:6: ERROR: Logical operator 'not' expects a comparison as its operand, not an integer expression.
```
