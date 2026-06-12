# Require: Float Variable Reference on the RHS

The right-hand side of a float `req` comparison may reference another float
variable, not just a float literal.

## Script
```cuentitos
--- variables
float health = 10.5
float threshold = 5.25
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
