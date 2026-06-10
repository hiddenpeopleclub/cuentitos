# Set: Self-Reference (Counter Pattern)

A `set` whose RHS references the LHS variable reads the current value,
evaluates the RHS, then assigns. This is the canonical counter pattern,
here accumulating a fractional step.

## Script
```cuentitos
--- variables
float counter = 0.0
---

set counter = counter + 1.5
set counter = counter + 1.5
set counter = counter + 1.5
Done.
```

## Input
```input
n
?
s
```

## Result
```result
START
Done.
counter: 4.5
END
```
