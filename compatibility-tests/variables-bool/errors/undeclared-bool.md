# Require Error: Truthiness Shortcut on an Undeclared Variable

A bare `req <name>` truthiness shortcut whose variable was never declared is a
parse-time error, just like any other reference to an undefined variable.

## Script
```cuentitos
--- variables
bool door_open = true
---

Line.
  req window_open
```

## Input
```input
s
```

## Result
```result
undeclared-bool.cuentitos:6: ERROR: Undefined variable: 'window_open'.
```
