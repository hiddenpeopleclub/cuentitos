# Require Error: Malformed Expression

A `req` whose RHS is a syntactically incomplete expression — here, a trailing
`+` with no right operand — is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x > 5 +
```

## Input
```input
s
```

## Result
```result
malformed-req-expression.cuentitos:6: ERROR: Malformed expression in 'req': 'x > 5 +'.
```
