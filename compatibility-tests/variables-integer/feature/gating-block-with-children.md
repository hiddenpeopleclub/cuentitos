# Require: Gating a Block with Children

When a `req` fails, the parent block **and every one of its descendants** are
skipped — not just the line the `req` is directly attached to.

## Script
```cuentitos
--- variables
int visited = 0
---

You stand in the hallway.
The old house.
  req visited = 1
  You remember being here.
  The smell is familiar.
You leave the hallway.
```

## Input
```input
s
```

## Result
```result
START
You stand in the hallway.
You leave the hallway.
END
```
