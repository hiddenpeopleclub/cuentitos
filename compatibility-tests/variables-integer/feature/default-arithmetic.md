# Default Using Arithmetic Over Earlier Variables

A default expression may use `+`, `-`, `*`, `/` and parentheses over earlier variables and literals.

## Script
```cuentitos
--- variables
int a = 5
int b = a + 5
int c = (a + b) * 2
int d = b - a
int e = b / a
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
a: 5
b: 10
c: 30
d: 5
e: 2
This is the story.
END
```
