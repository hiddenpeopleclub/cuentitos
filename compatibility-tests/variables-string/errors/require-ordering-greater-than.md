# Require Error: Greater-Than on String Values

Strings have no ordering — only equality (`=`) and inequality (`!=`) are
defined for them. Using `>` on a string is a parse-time error, mirroring the
bool rule (see `variables-bool/errors/ordering-greater-than.md`).

## Script
```cuentitos
--- variables
string name = "Aria"
---

Line.
  req name > "Aaa"
```

## Input
```input
s
```

## Result
```result
require-ordering-greater-than.cuentitos:6: ERROR: Ordering operator '>' is not supported on string values.
```
