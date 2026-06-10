# Error: Bool Default References A Non-Bool Variable

A bool default that references a variable of a different type (here an `int`)
is a type error.

## Script
```cuentitos
--- variables
int count = 3
bool b = count
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-references-non-bool.cuentitos:3: ERROR: Type mismatch: default for bool 'b' must be a bool, but 'count' is int.
```
