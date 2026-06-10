# Require: Bool Truthiness Shortcut

A bare `req <bool_var>` is a truthiness shortcut: it passes when the variable's
value is `true` and fails when it is `false`. No comparison operator is needed.

## Script
```cuentitos
--- variables
bool door_open = true
bool window_open = false
---

The door is open.
  req door_open
The window is open.
  req window_open
```

## Input
```input
s
```

## Result
```result
START
The door is open.
END
```
