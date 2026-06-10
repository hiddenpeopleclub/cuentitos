# Require: Bool Truthiness Combined with `and`

Bool truthiness shortcuts combine with `and`. The parent block is shown only
when both operands are `true`.

## Script
```cuentitos
--- variables
bool has_key = true
bool door_unlocked = true
bool guard_asleep = false
---

You slip inside.
  req has_key and door_unlocked
You are caught.
  req has_key and guard_asleep
```

## Input
```input
s
```

## Result
```result
START
You slip inside.
END
```
