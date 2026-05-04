# Logical Operator Error: Missing Operand for NOT

A `req` condition with `NOT` and no operand following it is a parse-time
error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req NOT
```

## Input
```input
s
```

## Result
```result
logical-missing-operand-not.cuentitos:6: ERROR: Missing operand for 'NOT' in 'req': 'NOT'.
```
