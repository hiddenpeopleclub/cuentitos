# Logical Operator Error: Missing Right Operand for OR

A `req` condition with `OR` and no right operand is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x > 0 OR
```

## Input
```input
s
```

## Result
```result
logical-missing-right-operand-or.cuentitos:6: ERROR: Missing right operand for 'OR' in 'req': 'x > 0 OR'.
```
