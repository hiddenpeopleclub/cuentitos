# Whitespace Tolerance Around `=` in a Declaration

A float declaration tolerates arbitrary whitespace (or none) on either side of
the `=` token between the variable name and its default value.

## Script
```cuentitos
--- variables
float a=1.5
float b =2.5
float c= 3.5
float d   =   4.5
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
a: 1.5
b: 2.5
c: 3.5
d: 4.5
This is the story.
END
```
