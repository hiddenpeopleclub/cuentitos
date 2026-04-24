# Require: Variable Reference on the RHS

The right-hand side of a `req` comparison may reference another variable,
not just an integer literal.

## Script
```cuentitos
--- variables
int health = 10
int threshold = 5
---

Health exceeds the threshold.
  req health > threshold
Threshold exceeds health.
  req threshold > health
```

## Input
```input
s
```

## Result
```result
START
Health exceeds the threshold.
END
```
