# Set Error: Invalid Escape Sequence on the RHS

Only `\"`, `\n`, and `\\` are supported escapes. Any other backslash
sequence inside a `set` RHS literal is a parse-time error, mirroring the
string *default* rule (see `errors/invalid-escape.md`).

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = "a\qb"
Hello
```

## Input
```input
s
```

## Result
```result
set-invalid-escape.cuentitos:4: ERROR: Invalid escape sequence '\q' in string literal.
```
