# Edge Case: Nested Parentheses in a Default

Deeply nested parentheses in a default expression should evaluate correctly.

## Script
```cuentitos
--- variables
int a = ((1 + 2) * (3 + 4))
int b = (((10 - 5) * 2) + 1)
int c = ((((2)))) + (((3)))
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
a: 21
b: 11
c: 5
This is the story.
END
```
