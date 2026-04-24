# Multiple Variables In One Block

A `--- variables` block may declare multiple variables; order is preserved for `?`.

## Script
```cuentitos
--- variables
int a
int b = 1
int c = 2
int d
int e = b + c
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
a: 0
b: 1
c: 2
d: 0
e: 3
This is the story.
END
```
