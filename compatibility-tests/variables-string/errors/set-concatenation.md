# Set Error: Concatenation on the RHS

String concatenation is not supported in v1. Using `+` to join strings on
the RHS of a `set` is a parse-time error, mirroring the string *default*
rule (see `errors/default-uses-concatenation.md`). The `set`-expression
parser does not special-case `+` between strings; it simply fails to parse
the RHS and reports the generic malformed-`set` diagnostic, echoing the
offending expression.

## Script
```cuentitos
--- variables
string greeting = "Hi"
---
set greeting = "Hello, " + "world"
```

## Input
```input
s
```

## Result
```result
set-concatenation.cuentitos:4: ERROR: Malformed 'set' statement: '"Hello, " + "world"'. ('set' is reserved at the start of a line; indent or rephrase to use it in narrative text.)
```
