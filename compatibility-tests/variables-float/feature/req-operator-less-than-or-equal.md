# Require: Less Than Or Equal Operator on Floats

A `req` using `<=` passes when the float LHS is less than or equal to the
float RHS. Equality at the boundary is accepted.

## Script
```cuentitos
--- variables
float health = 10.0
---

You are at or below the threshold.
  req health <= 10.0
You are at or below a lower threshold.
  req health <= 9.0
```

## Input
```input
s
```

## Result
```result
START
You are at or below the threshold.
END
```
