# Logical Operator Error: Missing Right Operand for AND

A `req` condition with `and` and no right operand is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x > 0 and
```

## Input
```input
s
```

## Result
```result
logical-missing-right-operand-and.cuentitos:6: ERROR: Missing right operand for 'and' in 'req': 'x > 0 and'.
```
