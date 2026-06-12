# Require Error: Bare Int Literal on the RHS of a Float `req`

A `req` comparing a float variable accepts only a float expression on the
other side. There is no implicit int-to-float coercion (the same rule the
float *defaults* and *sets* enforce, see `errors/cross-type-default.md` and
`errors/set-cross-type-int-rhs.md`), so a bare integer literal such as `5`
(no decimal point — it is an int literal, not a float literal) on the RHS of
a float comparison is a parse-time type-mismatch error. The diagnostic names
both sides of the rejected comparison.

## Script
```cuentitos
--- variables
float temp = 20.5
---

The reading holds.
  req temp > 5
```

## Input
```input
s
```

## Result
```result
req-cross-type-int-literal-rhs.cuentitos:6: ERROR: Type mismatch: cannot compare float 'temp' with int '5'.
```
