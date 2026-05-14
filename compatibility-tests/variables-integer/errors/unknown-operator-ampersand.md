# Require Error: Ampersand Used Instead of `and`

A `req` containing `&` (or `&&`) — the most common typo for `and` — is
rejected as an unknown operator. The message must not over-claim
"unknown comparison operator", since `&` isn't a comparison operator.

## Script
```cuentitos
--- variables
int x = 5
int y = 5
---

Line.
  req x > 0 & y > 0
```

## Input
```input
s
```

## Result
```result
unknown-operator-ampersand.cuentitos:7: ERROR: Unknown operator '&' in 'req'.
```
