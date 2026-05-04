# Logical Operator Error: Missing Left Operand for OR

A `req` condition with `OR` and no left operand is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req OR x > 0
```

## Input
```input
s
```

## Result
```result
logical-missing-left-operand-or.cuentitos:6: ERROR: Missing left operand for 'OR' in 'req': 'OR x > 0'.
```
