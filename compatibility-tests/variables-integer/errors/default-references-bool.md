# Error: Int Default References A Bool Variable

An `int` default that references an earlier variable of a different type (here
a `bool`) is a type error, not a missing or forward reference — the bool is
declared on the previous line. The diagnostic is symmetric with the bool
folder's wrong-kind path.

## Script
```cuentitos
--- variables
bool flag = true
int x = flag
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-references-bool.cuentitos:3: ERROR: Type mismatch: default for int 'x' must be a int, but 'flag' is bool.
```
