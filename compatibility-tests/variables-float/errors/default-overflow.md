# Error: Float Overflow in a Default (Constant-Folded)

Defaults are evaluated at parse time. A product whose magnitude exceeds the
largest finite `f64` overflows to infinity; rather than silently storing an
infinity, the engine reports a parse-time error. Each operand here is `1e200`
(written in the required plain decimal form, since exponent notation is not
allowed), and their product `1e400` is beyond the float range.

## Script
```cuentitos
--- variables
float boom = 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0 * 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-overflow.cuentitos:2: ERROR: Float overflow in default expression for 'boom'.
```
