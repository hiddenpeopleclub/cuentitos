# Require: Negative Float Literal on the RHS

A float `req` may use a negative float literal on the right-hand side. The
unary minus is part of the literal value (or the expression), and comparisons
are performed on signed floats — values below zero compare correctly against
both negative and zero thresholds.

## Script
```cuentitos
--- variables
float balance = -5.5
---

Balance is above the lower bound.
  req balance > -10.0
Balance is at or below zero.
  req balance <= 0.0
Balance equals negative five and a half.
  req balance = -5.5
Balance is not negative ten.
  req balance != -10.0
Balance is above zero.
  req balance > 0.0
Balance is below negative ten.
  req balance < -10.0
```

## Input
```input
s
```

## Result
```result
START
Balance is above the lower bound.
Balance is at or below zero.
Balance equals negative five and a half.
Balance is not negative ten.
END
```
