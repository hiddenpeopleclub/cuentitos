# Set: Arithmetic Expression on the RHS

The right-hand side of a `set` may be a full arithmetic expression
(`+ - * /`, parentheses, variables, literals), evaluated at runtime
before the assignment.

## Script
```cuentitos
--- variables
int a = 5
int b = 3
int c = 0
---

set c = (a + b) * 2
After.
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
After.
a: 5
b: 3
c: 16
END
```
