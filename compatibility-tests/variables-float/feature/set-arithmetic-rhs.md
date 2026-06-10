# Set: Arithmetic Expression on the RHS

The right-hand side of a `set` may be a full arithmetic expression
(`+ - * /`, parentheses, variables, float literals), evaluated at runtime
before the assignment. An integer-valued result still renders with a
fractional digit (`16.0`).

## Script
```cuentitos
--- variables
float a = 5.0
float b = 3.0
float c = 0.0
---

set c = (a + b) * 2.0
After.
```

## Input
```input
n
?
s
```

## Result
```result
START
After.
a: 5.0
b: 3.0
c: 16.0
END
```
