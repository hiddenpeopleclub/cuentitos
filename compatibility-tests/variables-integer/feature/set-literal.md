# Set: Literal RHS Updates the Variable

A `set <var> = <literal>` statement at the top level updates the named
variable's runtime value. A subsequent `?` reflects the new value.

## Script
```cuentitos
--- variables
int x = 0
---

Before.
set x = 7
After.
```

## Input
```input
n
?
n
?
s
```

## Result
```result
START
Before.
x: 0
After.
x: 7
END
```
