# Require: Less Than Operator

A `req` using `<` passes when the LHS is strictly less than the RHS and fails
otherwise.

## Script
```cuentitos
--- variables
int health = 10
---

You are below capacity.
  req health < 100
You are critically low.
  req health < 5
```

## Input
```input
s
```

## Result
```result
START
You are below capacity.
END
```
