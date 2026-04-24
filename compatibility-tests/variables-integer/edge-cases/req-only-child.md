# Require Edge Case: `req` Is the Only Child of a Block

When a `req` is a block's only child, the behavior is unchanged: the parent
is shown when the `req` passes and skipped when it fails.

## Script
```cuentitos
--- variables
int x = 1
---

Shown: only child is a passing req.
  req x = 1
Hidden: only child is a failing req.
  req x = 0
Shown: also only child, passing req.
  req x != 0
```

## Input
```input
s
```

## Result
```result
START
Shown: only child is a passing req.
Shown: also only child, passing req.
END
```
