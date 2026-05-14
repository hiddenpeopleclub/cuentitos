# Require Error: Double Equals in Comparison

Cuentitos spells equality as a single `=` in `req` (and assignment as
a single `=` in `set`). `==` — the most common typo from authors used
to C/Python/JavaScript — gets a dedicated diagnostic so the message
names the right operator instead of falling through to the generic
"malformed expression" error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x == 5
```

## Input
```input
s
```

## Result
```result
req-double-equals.cuentitos:6: ERROR: Use '=' for equality, not '=='.
```
