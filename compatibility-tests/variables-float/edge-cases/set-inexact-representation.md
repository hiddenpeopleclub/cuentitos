# Edge Case: Set Producing an Inexact Binary Result

Most decimal fractions are not exactly representable in IEEE-754 binary, so a
sum like `0.1 + 0.2` computed inside a `set` is not exactly `0.3`. The engine
renders the true stored value using its shortest decimal form that round-trips
back to the same `f64` (never scientific notation), which for `0.1 + 0.2` is
`0.30000000000000004`.

## Script
```cuentitos
--- variables
float x = 0.0
---
set x = 0.1 + 0.2
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
x: 0.30000000000000004
END
```
