# Negative Literal Default

Integer defaults may use negative literals.

## Script
```cuentitos
--- variables
int a = -5
int b = -10 + 3
int c = -(2 + 3)
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
a: -5
b: -7
c: -5
This is the story.
END
```
