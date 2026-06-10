# Set: Multi-Read RHS Uses Pre-Assignment Value

When a `set` RHS references the LHS variable more than once, **every** read
sees the value before the assignment. The RHS is evaluated to a single
float first, and only then assigned to the LHS. There is no left-to-right
write-before-read interleaving.

For `set x = x * 2.0 + x` with `x = 3.0`, the result must be `9.0`
(`3.0*2.0 + 3.0`), not `12.0` (which would be `3.0*2.0 = 6.0`, then
`6.0 + 6.0`). The latter would imply the first read of `x` updates the
variable before the second read.

## Script
```cuentitos
--- variables
float x = 3.0
---

set x = x * 2.0 + x
Done.
```

## Input
```input
n
?
s
```

## Result
```result
START
Done.
x: 9.0
END
```
