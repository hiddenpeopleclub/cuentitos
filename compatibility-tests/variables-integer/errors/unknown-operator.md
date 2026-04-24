# Require Error: Unknown Comparison Operator

A `req` using a symbol that is not one of the supported comparison operators
(`>`, `<`, `>=`, `<=`, `=`, `!=`) is a parse-time error.

## Script
```cuentitos
--- variables
int x = 5
---

Line.
  req x ~ 5
```

## Input
```input
s
```

## Result
```result
unknown-operator.cuentitos:6: ERROR: Unknown comparison operator: '~'.
```
