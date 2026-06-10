# Require: Multiple Bool `req` Siblings Act as Implicit AND

When a block has multiple `req` children, **all** of them must pass for the
parent to be shown. If any single bool `req` fails, the parent is skipped.

## Script
```cuentitos
--- variables
bool a = true
bool b = true
bool c = false
---

Both conditions pass.
  req a
  req b
One condition fails.
  req a
  req c
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
