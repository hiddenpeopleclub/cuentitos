# Logical Operator Error: Missing Left Operand for OR

A `req` condition with `or` and no left operand is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req or x > 0
```

## Input
```input
s
```

## Result
```result
logical-missing-left-operand-or.cuentitos:6: ERROR: Missing left operand for 'or' in 'req': 'or x > 0'.
```
