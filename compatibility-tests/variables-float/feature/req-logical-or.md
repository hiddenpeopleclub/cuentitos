# Require: Logical OR (Float Conditions)

A float `req` using `or` passes when at least one of the comparisons is true
and fails only when both are false.

## Script
```cuentitos
--- variables
float health = 10.0
float shield = 0.0
float armor = 0.0
---

You can act.
  req health > 0.0 or shield > 0.0
You are powerless.
  req shield > 0.0 or armor > 0.0
```

## Input
```input
s
```

## Result
```result
START
You can act.
END
```
