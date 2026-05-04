# Logical Operator Error: Missing Left Operand for AND

A `req` condition with `AND` and no left operand is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req AND x > 0
```

## Input
```input
s
```

## Result
```result
logical-missing-left-operand-and.cuentitos:6: ERROR: Missing left operand for 'AND' in 'req': 'AND x > 0'.
```
