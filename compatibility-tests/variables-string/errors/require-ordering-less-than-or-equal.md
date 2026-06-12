# Require Error: Less-Than-or-Equal on String Values

Strings have no ordering, so `<=` is undefined for them. Using it in a `req`
is a parse-time error.

## Script
```cuentitos
--- variables
string name = "Aria"
---

Line.
  req name <= "Zzz"
```

## Input
```input
s
```

## Result
```result
require-ordering-less-than-or-equal.cuentitos:6: ERROR: Ordering operator '<=' is not supported on string values.
```
