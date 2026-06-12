# Default Error: Float Overflow in a Literal

A float *default* whose value is a single literal beyond the largest finite
`f64` parses to infinity. Like the constant-folded product overflow in
`default-overflow.md`, the engine rejects it at parse time rather than
silently storing the infinity — a lone overflowing literal must not slip
through where `1e200 * 1e200` is rejected.

The literal here (`1e320`, written in the required plain decimal form since
exponent notation is not allowed) is beyond the float range.

## Script
```cuentitos
--- variables
float boom = 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-literal-overflow.cuentitos:2: ERROR: Float overflow in default expression for 'boom'.
```
