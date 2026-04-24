# Require: Multiple `req` Siblings Act as Implicit AND

When a block has multiple `req` children, **all** of them must pass for the
parent to be shown. If any single `req` fails, the parent is skipped.

## Script
```cuentitos
--- variables
int x = 5
---

Both conditions pass.
  req x > 0
  req x < 10
One condition fails.
  req x > 0
  req x > 100
All three pass.
  req x >= 5
  req x <= 5
  req x != 0
```

## Input
```input
s
```

## Result
```result
START
Both conditions pass.
All three pass.
END
```
