# Set Runtime Error: Division by Zero

Arithmetic inside a `set` expression is evaluated at runtime. Dividing by a
variable whose current value is zero produces a runtime error when the `set`
is reached, rather than an IEEE infinity — consistent with how float
*defaults* reject division by zero at parse time (see
`errors/division-by-zero.md`).

The divisor is a *variable reference* rather than a literal `0.0` because
literal-only arithmetic in defaults is constant-folded at parse time. The
spec deliberately does not fold variable references inside `set`, even when
the variable is never reassigned, so the division by zero surfaces at runtime.

The error message appears interleaved with story output on the same stream
(stdout); content emitted before the error is preserved, and the engine
halts after printing the error — `END` is not emitted.

## Script
```cuentitos
--- variables
float divisor = 0.0
float result
---
set result = 10.0 / divisor
Hello
```

## Input
```input
s
```

## Result
```result
START
set-runtime-division-by-zero.cuentitos:5: RUNTIME ERROR: Division by zero.
```
