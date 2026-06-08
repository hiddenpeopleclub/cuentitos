# Multiple Variables In One Block

A `--- variables` block may declare multiple float variables; declaration
order is preserved for `?`. Variables with no default initialize to `0.0`.

## Script
```cuentitos
--- variables
float a
float b = 1.5
float c = 2.5
float d
float e = b + c
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
a: 0.0
b: 1.5
c: 2.5
d: 0.0
e: 4.0
This is the story.
END
```
