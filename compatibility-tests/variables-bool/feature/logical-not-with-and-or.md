# Require: Bool `not` Mixed with `and` / `or`

A negated truthiness shortcut combines with `and` and `or` in the same
expression. `not` binds to its single operand before `and`/`or` apply.

## Script
```cuentitos
--- variables
bool alarm_on = false
bool door_open = true
---

Safe to move.
  req not alarm_on and door_open
Cannot move.
  req not door_open or alarm_on
```

## Input
```input
s
```

## Result
```result
START
Safe to move.
END
```
