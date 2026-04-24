# Require: Less Than Or Equal Operator

A `req` using `<=` passes when the LHS is less than or equal to the RHS.
Equality at the boundary is accepted.

## Script
```cuentitos
--- variables
int health = 10
---

You are at or below the threshold.
  req health <= 10
You are at or below a lower threshold.
  req health <= 9
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
