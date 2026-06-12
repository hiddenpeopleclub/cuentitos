# Require: Gating a Single-Line Block (Float Condition)

A float `req` placed under a single-line text block gates just that block.
When the `req` fails, only the parent line is skipped; surrounding siblings
are unaffected.

## Script
```cuentitos
--- variables
float door_open = 1.0
---

You approach the house.
The door is open.
  req door_open = 1.0
The door is locked.
  req door_open = 0.0
You step inside.
```

## Input
```input
s
```

## Result
```result
START
You approach the house.
The door is open.
You step inside.
END
```
