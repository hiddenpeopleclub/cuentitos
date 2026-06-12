# Require: Greater Than Operator on Floats

A `req` using `>` passes when the float LHS is strictly greater than the float
RHS and fails otherwise. The parent block is shown only when the `req` passes.

## Script
```cuentitos
--- variables
float health = 10.5
---

You are alive.
  req health > 0.0
You are unstoppable.
  req health > 100.0
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
