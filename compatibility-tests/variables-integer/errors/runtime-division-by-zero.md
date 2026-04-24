# Require Runtime Error: Division by Zero

Arithmetic inside a `req` expression is evaluated at runtime. Dividing by a
variable whose current value is zero produces a runtime error when the `req`
is reached.

## Script
```cuentitos
--- variables
int x = 10
int zero = 0
---

Before the gated block.
Gated block.
  req x > 10 / zero
```

## Input
```input
s
```

## Result
```result
START
Before the gated block.
runtime-division-by-zero.cuentitos:8: RUNTIME ERROR: Division by zero.
```
