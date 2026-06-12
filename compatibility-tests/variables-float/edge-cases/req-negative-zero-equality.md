# Require Edge Case: Negative Zero Compares Equal to Positive Zero

IEEE-754 stores `-0.0` and `+0.0` as distinct bit patterns (and the engine
*renders* them distinctly), but the comparison operators treat them as equal:
`-0.0 = 0.0` is true, and `-0.0 < 0.0` is false. A `req` follows IEEE
comparison semantics, so the sign of zero does not affect ordering or
equality even though it affects display.

## Script
```cuentitos
--- variables
float neg_zero = -0.0
float pos_zero = 0.0
---

Negative zero equals the positive zero literal.
  req neg_zero = 0.0
Negative zero equals the positive zero variable.
  req neg_zero = pos_zero
Negative zero is not strictly less than zero.
  req neg_zero < 0.0
```

## Input
```input
s
```

## Result
```result
START
Negative zero equals the positive zero literal.
Negative zero equals the positive zero variable.
END
```
