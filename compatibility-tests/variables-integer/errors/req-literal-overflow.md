# Require Error: Integer Literal Overflow

A `req` expression containing an integer literal that exceeds the integer
range is a parse-time error. Previously this folded into the generic
"Malformed expression" diagnostic; surface a dedicated overflow message
so the author knows the literal — not the surrounding syntax — is the
problem.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x > 99999999999999999999
```

## Input
```input
s
```

## Result
```result
req-literal-overflow.cuentitos:6: ERROR: Integer overflow in 'req' expression: literal '99999999999999999999' exceeds the integer range.
```
