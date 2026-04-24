# Set Runtime Error: Integer Overflow

Arithmetic inside a `set` expression that overflows the integer type at
runtime produces a runtime error when the `set` is reached.

`max_val` (the i64 max) is a variable reference rather than a literal so the
overflowing `max_val + 1` cannot be constant-folded at parse time. The spec
deliberately does not fold variable references inside `req`/`set`, even
when the variable is never `set`, so the overflow surfaces at runtime
rather than as a parse-time error (see `errors/overflow.md` for the
constant-folded counterpart).

The error message appears interleaved with story output on the same stream
(stdout); content emitted before the error is preserved, and the engine
halts after printing the error — `END` is not emitted.

## Script
```cuentitos
--- variables
int max_val = 9223372036854775807
int result
---
set result = max_val + 1
Hello
```

## Input
```input
s
```

## Result
```result
START
set-runtime-overflow.cuentitos:5: RUNTIME ERROR: Integer overflow.
```
