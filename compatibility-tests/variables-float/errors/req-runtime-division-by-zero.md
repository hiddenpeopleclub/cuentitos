# Require Runtime Error: Float Division by Zero

Arithmetic inside a float `req` expression is evaluated at runtime. Dividing
by a variable whose current value is zero produces a runtime error when the
`req` is reached, rather than an IEEE infinity — consistent with how float
*defaults* and *sets* reject division by zero (see `errors/division-by-zero.md`
and `errors/set-runtime-division-by-zero.md`).

The divisor is a *variable reference* rather than a literal `0.0` because
literal-only arithmetic in defaults is constant-folded at parse time. The
spec deliberately does not fold variable references inside `req`/`set`, even
when the variable is never `set`, so the division by zero surfaces at runtime.

The error message appears interleaved with story output on the same stream
(stdout); content emitted before the error is preserved, and the engine halts
after printing the error — `END` is not emitted.

## Script
```cuentitos
--- variables
float x = 10.0
float zero = 0.0
---

Before the gated block.
Gated block.
  req x > 10.0 / zero
```

## Input
```input
s
```

## Result
```result
START
Before the gated block.
req-runtime-division-by-zero.cuentitos:8: RUNTIME ERROR: Division by zero.
```
