# Require: Less Than Operator on Floats

A `req` using `<` passes when the float LHS is strictly less than the float
RHS and fails otherwise.

## Script
```cuentitos
--- variables
float health = 10.0
---

You are below the cap.
  req health < 100.0
You are below the floor.
  req health < 1.0
```

## Input
```input
s
```

## Result
```result
START
You are below the cap.
END
```
