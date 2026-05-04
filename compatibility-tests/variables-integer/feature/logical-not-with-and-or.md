# Require: NOT Combined with AND and OR

`NOT` binds tightest, then `AND`, then `OR`. Combinations of all three follow
that precedence and produce the expected truth values.

## Script
```cuentitos
--- variables
int health = 10
int shield = 0
int armor = 5
---

Alive but exposed.
  req health > 0 AND NOT shield > 0
Dangerous combo.
  req NOT health > 0 OR armor > 0
Impossible.
  req NOT health > 0 AND armor > 0
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
