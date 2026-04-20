# Require: Greater Than Or Equal Operator

A `req` using `>=` passes when the LHS is greater than or equal to the RHS.
Equality at the boundary is accepted.

## Script
```cuentitos
--- variables
int health = 10
---

You are at or above the threshold.
  req health >= 10
You are at or above a higher threshold.
  req health >= 11
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
