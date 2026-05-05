# Require: Logical AND

A `req` using `and` passes only when both comparisons are true. The parent
block is shown only when the combined expression evaluates to true.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
int armor = 0
---

Defended.
  req health > 0 and shield > 0
Exposed.
  req health > 0 and armor > 0
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
