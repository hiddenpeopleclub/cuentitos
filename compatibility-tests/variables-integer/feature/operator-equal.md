# Require: Equal Operator

A `req` using `=` passes when the LHS equals the RHS.

## Script
```cuentitos
--- variables
int health = 10
---

You match the target value.
  req health = 10
You match a different value.
  req health = 5
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
