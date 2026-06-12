# Set: Literal RHS Updates the Variable

A `set <var> = "<literal>"` statement at the top level updates the named
string variable's runtime value. A subsequent `?` reflects the new value,
re-rendered double-quoted.

## Script
```cuentitos
--- variables
string name = "Aria"
---

Before.
set name = "Brenn"
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
name: "Aria"
After.
name: "Brenn"
END
```
