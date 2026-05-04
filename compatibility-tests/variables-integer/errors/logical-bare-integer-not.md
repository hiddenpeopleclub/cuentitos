# Logical Operator Error: NOT Applied to a Bare Integer Expression

Logical operators combine **comparisons**, not bare integer expressions. A
`req` whose `NOT` operand is a plain integer term (no comparison operator)
is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
---

Line.
  req NOT health
```

## Input
```input
s
```

## Result
```result
logical-bare-integer-not.cuentitos:6: ERROR: Logical operator 'NOT' expects a comparison as its operand, not an integer expression.
```
