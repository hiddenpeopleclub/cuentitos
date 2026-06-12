# Require Edge Case: Whitespace Inside a String Is Significant

String equality compares the full character content, so leading or trailing
whitespace inside the literal counts. `"Aria"` and `"Aria "` are different
values.

## Script
```cuentitos
--- variables
string name = "Aria"
---

No trailing space matches.
  req name = "Aria"
Trailing space does not match.
  req name = "Aria "
```

## Input
```input
s
```

## Result
```result
START
No trailing space matches.
END
```
