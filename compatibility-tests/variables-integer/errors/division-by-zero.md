# Error: Division By Zero in a Default (Constant-Folded)

Because defaults are evaluated at parse time, division by zero must be a parse-time error.

## Script
```cuentitos
--- variables
int a = 10 / 0
---

This is the story.
```

## Input
```input
s
```

## Result
```result
division-by-zero.cuentitos:2: ERROR: Division by zero in default expression for 'a'.
```
