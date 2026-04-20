# Require: Not Equal Operator

A `req` using `!=` passes when the LHS is not equal to the RHS.

## Script
```cuentitos
--- variables
int health = 10
---

You are not at zero.
  req health != 0
You are not at ten.
  req health != 10
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
