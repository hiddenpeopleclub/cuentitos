# Require: Bool Truthiness Combined with `or`

Bool truthiness shortcuts combine with `or`. The parent block is shown when at
least one operand is `true` and is skipped only when both are `false`.

## Script
```cuentitos
--- variables
bool has_key = true
bool has_lockpick = false
bool has_crowbar = false
---

You can enter.
  req has_key or has_lockpick
You are stuck outside.
  req has_lockpick or has_crowbar
```

## Input
```input
s
```

## Result
```result
START
You can enter.
END
```
