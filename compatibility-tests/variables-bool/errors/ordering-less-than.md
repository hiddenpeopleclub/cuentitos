# Require Error: Less-Than on Bool Values

Bools have no ordering — only equality (`=`) and inequality (`!=`) are
defined for them. Using `<` on a bool is a parse-time error.

## Script
```cuentitos
--- variables
bool door_open = true
---

Line.
  req door_open < true
```

## Input
```input
s
```

## Result
```result
ordering-less-than.cuentitos:6: ERROR: Ordering operator '<' is not supported on bool values.
```
