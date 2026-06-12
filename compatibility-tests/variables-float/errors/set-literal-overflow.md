# Set Error: Float Overflow in a Literal

A single float *literal* whose magnitude exceeds the largest finite `f64`
parses to infinity. Rather than silently storing that infinity, the engine
rejects it at parse time — consistent with how a constant-folded product
overflow is rejected (see `set-runtime-overflow.md`) and how a float
*default* literal overflow is rejected (see `default-overflow.md`).

The literal here (`1e320`, written in the required plain decimal form since
exponent notation is not allowed) is beyond the float range. Because it is a
lone literal — not a product of variables — the overflow is caught while
parsing the `set`, so `START` is never emitted.

## Script
```cuentitos
--- variables
float result
---
set result = 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0
Hello
```

## Input
```input
s
```

## Result
```result
set-literal-overflow.cuentitos:4: ERROR: Float overflow in 'set' expression for 'result'.
```
