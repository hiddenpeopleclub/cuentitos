# Require Error: Unknown Operator

A `req` containing a symbol the tokenizer doesn't recognize as part of
any grammar token is a parse-time error. This covers both unsupported
comparison operators (e.g. `~`) and stray symbols that aren't operators
at all (e.g. `&`, `|`); the diagnostic must say "unknown operator", not
"unknown comparison operator".

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
unknown-operator.cuentitos:6: ERROR: Unknown operator '~' in 'req'.
```
