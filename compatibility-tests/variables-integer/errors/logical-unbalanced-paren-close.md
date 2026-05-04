# Logical Operator Error: Unbalanced Parentheses — Closing Without Opening

A `req` condition with a closing parenthesis but no matching opening
parenthesis is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x > 0 AND x < 10)
```

## Input
```input
s
```

## Result
```result
logical-unbalanced-paren-close.cuentitos:6: ERROR: Unbalanced parentheses in 'req': 'x > 0 AND x < 10)'.
```
