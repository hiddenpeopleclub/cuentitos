# Require: Multiple `req` Siblings Act as Implicit AND (Float)

When a block has multiple float `req` children, **all** of them must pass for
the parent to be shown. If any single `req` fails, the parent is skipped.

## Script
```cuentitos
--- variables
float x = 5.5
---

Both conditions pass.
  req x > 0.0
  req x < 10.0
One condition fails.
  req x > 0.0
  req x > 100.0
All three pass.
  req x >= 5.5
  req x <= 5.5
  req x != 0.0
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
