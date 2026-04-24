# Set: Multi-Read RHS Uses Pre-Assignment Value

When a `set` RHS references the LHS variable more than once, **every** read
sees the value before the assignment. The RHS is evaluated to a single
integer first, and only then assigned to the LHS. There is no left-to-right
write-before-read interleaving.

For `set x = x * 2 + x` with `x = 3`, the result must be `9` (`3*2 + 3`),
not `12` (which would be `3*2 = 6`, then `6 + 6`). The latter would imply
the first read of `x` updates the variable before the second read.

## Script
```cuentitos
--- variables
int x = 3
---

set x = x * 2 + x
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
x: 9
END
```
