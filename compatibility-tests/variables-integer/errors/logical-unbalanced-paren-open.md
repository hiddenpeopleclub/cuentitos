# Logical Operator Error: Unbalanced Parentheses — Opening Without Closing

A `req` condition with an opening parenthesis but no matching closing
parenthesis is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req (x > 0 AND x < 10
```

## Input
```input
s
```

## Result
```result
logical-unbalanced-paren-open.cuentitos:6: ERROR: Unbalanced parentheses in 'req': '(x > 0 AND x < 10'.
```
