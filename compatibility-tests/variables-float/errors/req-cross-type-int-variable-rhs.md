# Require Error: Int Variable on the RHS of a Float `req`

A `req` comparing a float variable accepts only a float expression on the
other side. There is no implicit int-to-float coercion (the same rule the
float *defaults* and *sets* enforce, see `errors/cross-type-default.md` and
`errors/set-cross-type-int-rhs.md`), so referencing an int variable on the
RHS of a float comparison is a parse-time type-mismatch error.

## Script
```cuentitos
--- variables
int count = 3
float temp = 20.5
---

The reading holds.
  req temp > count
```

## Input
```input
s
```

## Result
```result
req-cross-type-int-variable-rhs.cuentitos:7: ERROR: Type mismatch: cannot compare float 'temp' with int 'count'.
```
