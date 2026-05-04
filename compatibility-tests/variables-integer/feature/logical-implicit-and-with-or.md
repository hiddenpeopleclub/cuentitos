# Require: Implicit AND Across Sibling `req`s Composes with Inline OR

Sibling `req`s under the same parent are implicitly ANDed. When one of those
siblings uses an inline `or`, the `or` is evaluated as a single condition
that participates in the implicit AND with the other siblings.

The first parent block has both gates pass. The second parent block has the
inline-`or` sibling pass but the second sibling fail — the implicit AND
across siblings then makes the parent fail, even though the `or` sibling
succeeds on its own.

## Script
```cuentitos
--- variables
int health = 10
int shield = 0
int armor = 0
int mana = 5
---

Both gates pass.
  req health > 0 or shield > 0
  req mana > 0
One gate fails.
  req health > 0 or shield > 0
  req armor > 0
```

## Input
```input
s
```

## Result
```result
START
Both gates pass.
END
```
