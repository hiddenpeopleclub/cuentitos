# Set Runtime Error: Float Overflow

Arithmetic inside a `set` expression that overflows the float type to
infinity at runtime produces a runtime error when the `set` is reached,
rather than silently storing an infinity — consistent with how float
*defaults* reject overflow at parse time (see `errors/default-overflow.md`).

`huge` (`1e200`, written in the required plain decimal form) is a variable
reference rather than a literal so the overflowing `huge * huge` (`1e400`,
beyond the largest finite `f64`) cannot be constant-folded at parse time;
the overflow therefore surfaces at runtime.

The error message appears interleaved with story output on the same stream
(stdout); content emitted before the error is preserved, and the engine
halts after printing the error — `END` is not emitted.

## Script
```cuentitos
--- variables
float huge = 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0
float result
---
set result = huge * huge
Hello
```

## Input
```input
s
```

## Result
```result
START
set-runtime-overflow.cuentitos:5: RUNTIME ERROR: Float overflow.
```
