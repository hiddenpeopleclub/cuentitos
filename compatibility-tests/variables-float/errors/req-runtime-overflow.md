# Require Runtime Error: Float Overflow

Arithmetic inside a float `req` expression that overflows the float type to
infinity at runtime produces a runtime error when the `req` is reached, rather
than silently storing an infinity — consistent with how float *defaults* and
*sets* reject overflow (see `errors/default-overflow.md` and
`errors/set-runtime-overflow.md`).

`huge` (`1e200`, written in the required plain decimal form since exponent
notation is not allowed) is a variable reference rather than a literal, so the
overflowing `huge * huge` (`1e400`, beyond the largest finite `f64`) cannot be
constant-folded at parse time; the overflow therefore surfaces at runtime
while evaluating the comparison's left-hand side, before the `< 1.0` threshold
is ever consulted.

The error message appears interleaved with story output on the same stream
(stdout); content emitted before the error is preserved, and the engine halts
after printing the error — `END` is not emitted.

## Script
```cuentitos
--- variables
float huge = 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0
---

Before the gated block.
Gated block.
  req huge * huge < 1.0
```

## Input
```input
s
```

## Result
```result
START
Before the gated block.
req-runtime-overflow.cuentitos:7: RUNTIME ERROR: Float overflow.
```
