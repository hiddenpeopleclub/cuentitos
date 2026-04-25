# Set Expressions Do Not Fold Through Variable References

A `set` expression that mixes a literal with a variable reference must
be evaluated at runtime against the variable's *current* value, not the
variable's declared default. If the parser were to constant-fold the
default into the set expression, this script would overflow at parse
time — but the prior `set big = 1` makes `big + 2` a perfectly safe
runtime computation that yields `3`.

This is the negative twin of the `overflow-through-variable.md` test
on default expressions: defaults *do* fold through earlier references,
set expressions do *not*.

## Script
```cuentitos
--- variables
int big = 9223372036854775806
int result
---
set big = 1
set result = big + 2
Hello
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
Hello
big: 1
result: 3
END
```
