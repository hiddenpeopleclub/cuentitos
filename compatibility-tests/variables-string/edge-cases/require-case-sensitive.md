# Require Edge Case: String Equality Is Case-Sensitive

String comparison is exact and case-sensitive. `"Aria"` and `"aria"` are
distinct values, so a `req` that differs only in letter case does not pass.

## Script
```cuentitos
--- variables
string name = "Aria"
---

Exact case matches.
  req name = "Aria"
Different case does not match.
  req name = "aria"
```

## Input
```input
s
```

## Result
```result
START
Exact case matches.
END
```
