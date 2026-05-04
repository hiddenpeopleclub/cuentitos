# Require: Logical OR

A `req` using `OR` passes when at least one of the comparisons is true and
fails only when both are false.

## Script
```cuentitos
--- variables
int health = 10
int shield = 0
int armor = 0
---

You can act.
  req health > 0 OR shield > 0
You are powerless.
  req shield > 0 OR armor > 0
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
