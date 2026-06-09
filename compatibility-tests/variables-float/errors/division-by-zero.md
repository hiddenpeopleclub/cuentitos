# Error: Division By Zero in a Default (Constant-Folded)

Because defaults are evaluated at parse time, division by zero must be a
parse-time error rather than producing an IEEE infinity.

## Script
```cuentitos
--- variables
float a = 10.0 / 0.0
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
