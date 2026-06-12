# Require: Gating a Single-Line String-Conditioned Block

A string `req` placed under a single-line text block gates just that block.
When the `req` fails, only the parent line is skipped; surrounding siblings
are unaffected.

## Script
```cuentitos
--- variables
string door = "open"
---

You approach the house.
The door is open.
  req door = "open"
The door is locked.
  req door = "locked"
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
