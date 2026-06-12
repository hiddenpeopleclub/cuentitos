# Require: Greater Than Or Equal Operator on Floats

A `req` using `>=` passes when the float LHS is greater than or equal to the
float RHS. Equality at the boundary is accepted.

## Script
```cuentitos
--- variables
float health = 10.0
---

You are at or above the threshold.
  req health >= 10.0
You are at or above a higher threshold.
  req health >= 11.0
```

## Input
```input
s
```

## Result
```result
START
You are at or above the threshold.
END
```
