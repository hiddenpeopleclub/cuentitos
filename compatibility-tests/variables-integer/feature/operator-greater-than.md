# Require: Greater Than Operator

A `req` using `>` passes when the LHS is strictly greater than the RHS and
fails otherwise. The parent block is shown only when the `req` passes.

## Script
```cuentitos
--- variables
int health = 10
---

You are alive.
  req health > 0
You are unstoppable.
  req health > 100
```

## Input
```input
s
```

## Result
```result
START
You are alive.
END
```
