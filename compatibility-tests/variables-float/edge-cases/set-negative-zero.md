# Edge Case: Set Producing Negative Zero

IEEE-754 distinguishes `+0.0` from `-0.0`. A `set` whose RHS evaluates to
negative zero (here `0.0 * -1.0`) stores and renders `-0.0`, preserving the
sign of zero, even though the variable's prior value was a positive `1.0`.

## Script
```cuentitos
--- variables
float x = 1.0
---
set x = 0.0 * -1.0
Hello
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
Hello
x: -0.0
END
```
