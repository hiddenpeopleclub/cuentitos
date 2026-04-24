# Require: Negative Literal on the RHS

A `req` may use a negative integer literal on the right-hand side. The unary
minus is part of the literal value (or the expression), and comparisons are
performed on signed integers — values below zero compare correctly against
both negative and zero thresholds.

## Script
```cuentitos
--- variables
int balance = -5
---

Balance is above the lower bound.
  req balance > -10
Balance is at or below zero.
  req balance <= 0
Balance equals negative five.
  req balance = -5
Balance is not negative ten.
  req balance != -10
Balance is above zero.
  req balance > 0
Balance is below negative ten.
  req balance < -10
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
Balance equals negative five.
Balance is not negative ten.
END
```
