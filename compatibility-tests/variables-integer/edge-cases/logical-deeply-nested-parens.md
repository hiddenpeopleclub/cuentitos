# Edge Case: Deeply Nested Parentheses in `req` Conditions

Logical sub-expressions nested three or more levels deep should evaluate
correctly. Each gated line has at least three nested parenthesis pairs
mixing `AND`, `OR`, and `NOT`. Two of the three expressions are true and
one is false — the false case proves the engine actually evaluates the
nested structure rather than always succeeding.

## Script
```cuentitos
--- variables
int health = 10
int shield = 5
int armor = 0
int mana = 0
---

Triple nested AND, all true.
  req ((health > 0) AND ((shield > 0) AND (health < 100)))
Quad nested with one inner false.
  req (((health > 0 OR armor > 0) AND (mana > 0 OR armor > 0)))
Triple nested with NOT.
  req (((NOT armor > 0) AND health > 0))
```

## Input
```input
s
```

## Result
```result
START
Triple nested AND, all true.
Triple nested with NOT.
END
```
