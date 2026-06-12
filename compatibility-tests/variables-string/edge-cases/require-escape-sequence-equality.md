# Require Edge Case: Escape Sequences Compare by Decoded Value

A string literal's escape sequences are decoded before comparison, so a
variable holding a newline matches a literal written with `\n`, and does not
match the same text with a literal space instead.

## Script
```cuentitos
--- variables
string greeting = "hi\nthere"
---

Matches the escaped literal.
  req greeting = "hi\nthere"
Does not match a flattened version.
  req greeting = "hi there"
```

## Input
```input
s
```

## Result
```result
START
Matches the escaped literal.
END
```
