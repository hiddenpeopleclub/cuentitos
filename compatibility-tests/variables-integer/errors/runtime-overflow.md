# Require Runtime Error: Integer Overflow

Arithmetic inside a `req` expression that overflows the integer type at
runtime produces a runtime error when the `req` is reached.

`big` (the i64 max) is a variable reference rather than a literal so the
overflowing `big + 1` cannot be constant-folded at parse time. The spec
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
int big = 9223372036854775807
---

Before the gated block.
Gated block.
  req big < big + 1
```

## Input
```input
s
```

## Result
```result
START
Before the gated block.
runtime-overflow.cuentitos:7: RUNTIME ERROR: Integer overflow.
```
