# Require: Logical AND (Float Conditions)

A float `req` using `and` passes only when both comparisons are true. The
parent block is shown only when the combined expression evaluates to true.

## Script
```cuentitos
--- variables
float health = 10.0
float shield = 5.0
float armor = 0.0
---

Defended.
  req health > 0.0 and shield > 0.0
Exposed.
  req health > 0.0 and armor > 0.0
```

## Input
```input
s
```

## Result
```result
START
Defended.
END
```
