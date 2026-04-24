# Require Runtime Error: Integer Overflow

Arithmetic inside a `req` expression that overflows the integer type at
runtime produces a runtime error when the `req` is reached.

## Script
```cuentitos
--- variables
int big = 2147483647
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
RUNTIME ERROR: Integer overflow
```
