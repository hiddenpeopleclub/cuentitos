# Require: Parenthesized Arithmetic on the RHS

A `req` may use a parenthesized arithmetic expression on the right-hand
side. The `(` here sits after a comparison operator, so it goes straight
through the arithmetic operand parser — it isn't the parenthesized-LHS
shape that the boolean parser's lookahead disambiguates. Sub-expression
grouping must still respect operator precedence.

## Script
```cuentitos
--- variables
int health = 10
int shield = 3
int bonus = 2
---

Health beats shield plus the bonus (grouped on the right).
  req health > (shield + bonus)
Health beats the bonus times shield plus one.
  req health > (bonus * shield) + 1
Health does not beat shield times the bonus plus seven.
  req health > (shield * bonus) + 7
```

## Input
```input
s
```

## Result
```result
START
Health beats shield plus the bonus (grouped on the right).
Health beats the bonus times shield plus one.
END
```
