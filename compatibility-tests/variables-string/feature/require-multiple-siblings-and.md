# Require: Multiple String `req` Siblings Act as Implicit AND

When a block has multiple `req` children, **all** of them must pass for the
parent to be shown. If any single `req` fails, the parent is skipped.

## Script
```cuentitos
--- variables
string name = "Aria"
---

Both conditions pass.
  req name = "Aria"
  req name != "Brenn"
One condition fails.
  req name = "Aria"
  req name = "Brenn"
```

## Input
```input
s
```

## Result
```result
START
Both conditions pass.
END
```
