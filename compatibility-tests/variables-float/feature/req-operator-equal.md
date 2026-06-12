# Require: Equal Operator on Floats

A `req` using `=` passes when the float LHS equals the float RHS. Equality is
the exact IEEE-754 comparison of the stored `f64` values.

## Script
```cuentitos
--- variables
float ratio = 1.5
---

You match the target value.
  req ratio = 1.5
You match a different value.
  req ratio = 2.5
```

## Input
```input
s
```

## Result
```result
START
You match the target value.
END
```
