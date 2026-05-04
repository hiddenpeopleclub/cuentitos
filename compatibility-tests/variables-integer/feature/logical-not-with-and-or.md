# Require: NOT Combined with AND and OR

`not` binds tightest, then `and`, then `or`. Combinations of all three follow
that precedence and produce the expected truth values.

## Script
```cuentitos
--- variables
int health = 10
int shield = 0
int armor = 5
---

Alive but exposed.
  req health > 0 and not shield > 0
Dangerous combo.
  req not health > 0 or armor > 0
Impossible.
  req not health > 0 and armor > 0
```

## Input
```input
s
```

## Result
```result
START
Alive but exposed.
Dangerous combo.
END
```
