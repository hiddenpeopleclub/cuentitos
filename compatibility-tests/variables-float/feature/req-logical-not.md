# Require: Logical NOT (Float Condition)

A float `req` using `not` inverts the truth value of its comparison operand.
The parent block is shown only when the negated expression is true.

## Script
```cuentitos
--- variables
float health = 10.0
float shield = 0.0
---

Unshielded.
  req not shield > 0.0
Unhealthy.
  req not health > 0.0
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
