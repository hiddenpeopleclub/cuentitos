# Set Error: Unterminated String Literal on the RHS

A `set` RHS literal that opens a double quote but never closes it on the
same line is a parse-time error, mirroring the string *default* rule (see
`errors/unterminated-literal.md`).

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = "Brenn
Hello
```

## Input
```input
s
```

## Result
```result
set-unterminated-literal.cuentitos:4: ERROR: Unterminated string literal.
```
