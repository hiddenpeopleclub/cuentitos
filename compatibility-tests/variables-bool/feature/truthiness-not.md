# Require: Bool Truthiness Shortcut Negated with `not`

`req not <bool_var>` inverts the truthiness shortcut: it passes when the
variable is `false` and fails when it is `true`.

## Script
```cuentitos
--- variables
bool door_open = true
bool window_open = false
---

The door is shut.
  req not door_open
The window is shut.
  req not window_open
```

## Input
```input
s
```

## Result
```result
START
The window is shut.
END
```
