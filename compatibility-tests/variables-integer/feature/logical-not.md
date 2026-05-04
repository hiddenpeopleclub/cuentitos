# Require: Logical NOT

A `req` using `NOT` inverts the truth value of its comparison operand. The
parent block is shown only when the negated expression is true.

## Script
```cuentitos
--- variables
int health = 10
int shield = 0
---

Unshielded.
  req NOT shield > 0
Unhealthy.
  req NOT health > 0
```

## Input
```input
s
```

## Result
```result
START
Unshielded.
END
```
