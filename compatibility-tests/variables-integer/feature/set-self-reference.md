# Set: Self-Reference (Counter Pattern)

A `set` whose RHS references the LHS variable reads the current value,
evaluates the RHS, then assigns. This is the canonical counter pattern.

## Script
```cuentitos
--- variables
int counter = 0
---

set counter = counter + 1
set counter = counter + 1
set counter = counter + 1
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
counter: 3
END
```
