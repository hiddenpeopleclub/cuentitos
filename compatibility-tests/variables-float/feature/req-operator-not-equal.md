# Require: Not Equal Operator on Floats

A `req` using `!=` passes when the float LHS is not equal to the float RHS.

## Script
```cuentitos
--- variables
float ratio = 1.5
---

You are not at zero.
  req ratio != 0.0
You are not at one and a half.
  req ratio != 1.5
```

## Input
```input
s
```

## Result
```result
START
You are not at zero.
END
```
