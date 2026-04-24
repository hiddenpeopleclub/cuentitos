# Require: Gating a Single-Line Block

A `req` placed under a single-line text block gates just that block. When the
`req` fails, only the parent line is skipped; surrounding siblings are
unaffected.

## Script
```cuentitos
--- variables
int door_open = 1
---

You approach the house.
The door is open.
  req door_open = 1
The door is locked.
  req door_open = 0
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
